use proc_macro::TokenStream;
use quote::quote;
use syn::{
  parse::{self, Parse, ParseStream},
  parse_macro_input, parse_quote,
  punctuated::Punctuated,
  DeriveInput, Error, Path, Token,
};

use crate::helpers::add_value_cell;

struct LazyValueViewArgs {
  resolver: Path,
  value_type: Path,
  error_type: Option<Path>,
}

impl Parse for LazyValueViewArgs {
  fn parse(input: ParseStream) -> parse::Result<Self> {
    let vars = Punctuated::<Path, Token![,]>::parse_terminated(input)?;
    let mut vars_iter = vars.iter();
    let resolver = vars_iter
      .next()
      .ok_or_else(|| Error::new(input.span(), "Resolver function name expected"))?;
    let value_type = vars_iter
      .next()
      .ok_or_else(|| Error::new(input.span(), "Value type expected"))?;
    let error_type = vars_iter.next();
    if vars_iter.next().is_some() {
      return Err(Error::new(
        input.span(),
        "Unexpected parameter for lazy_liquid_value_view macro",
      ));
    }

    Ok(LazyValueViewArgs {
      resolver: resolver.to_owned(),
      value_type: value_type.to_owned(),
      error_type: error_type.map(|path| path.to_owned()),
    })
  }
}

pub fn eval_lazy_value_view_macro(args: TokenStream, input: TokenStream) -> TokenStream {
  let args = parse_macro_input!(args as LazyValueViewArgs);
  let mut input = parse_macro_input!(input as DeriveInput);
  let name = &input.ident;
  let generics = &input.generics;
  let resolver = &args.resolver;
  let value_type = &args.value_type;
  let error_type = args
    .error_type
    .unwrap_or_else(|| parse_quote!(liquid::Error));

  add_value_cell(&mut input.data, &args.value_type);

  let lazy_value_view_impl = quote!(
    #[async_trait]
    impl #generics lazy_liquid_value_view::LazyValueView for #name #generics {
      type Value = #value_type;
      type Error = #error_type;

      async fn resolve(&self) -> Result<&Self::Value, Self::Error> {
        self
          .value_cell
          .get_or_try_init(|| async { #resolver(&self).await })
          .await
      }

      fn get_resolved(&self) -> Option<&Self::Value> {
        self.value_cell.get()
      }
    }
  );

  let value_view_impl = quote!(
    impl #generics liquid::ValueView for #name #generics {
      fn as_debug(&self) -> &dyn std::fmt::Debug {
        self
          .as_value_sync()
          .map(|value| value.as_debug())
          .unwrap_or_else(|_| liquid::model::Value::Nil.as_debug())
      }

      fn render(&self) -> liquid::model::DisplayCow<'_> {
        self
          .as_value_sync()
          .map(|value| value.render())
          .unwrap_or_else(|_| liquid::model::Value::Nil.render())
      }

      fn source(&self) -> liquid::model::DisplayCow<'_> {
        self
          .as_value_sync()
          .map(|value| value.source())
          .unwrap_or_else(|_| liquid::model::Value::Nil.source())
      }

      fn type_name(&self) -> &'static str {
        self
          .as_value_sync()
          .map(|value| value.type_name())
          .unwrap_or_else(|_| liquid::model::Value::Nil.type_name())
      }

      fn query_state(&self, state: liquid::model::State) -> bool {
        self
          .as_value_sync()
          .map(|value| value.query_state(state))
          .unwrap_or_else(|_| liquid::model::Value::Nil.query_state(state))
      }

      fn to_kstr(&self) -> liquid::model::KStringCow<'_> {
        self
          .as_value_sync()
          .map(|value| value.to_kstr())
          .unwrap_or_else(|_| liquid::model::Value::Nil.to_kstr())
      }

      fn to_value(&self) -> liquid_core::Value {
        self
          .as_value_sync()
          .map(|value| value.to_value())
          .unwrap_or_else(|_| liquid::model::Value::Nil.to_value())
      }
    }
  );

  quote!(
    #input
    #lazy_value_view_impl
    #value_view_impl
  )
  .into()
}
