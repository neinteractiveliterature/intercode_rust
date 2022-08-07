extern crate proc_macro;
use drop_getter_method::DropGetterMethod;
use helpers::{add_value_cell, get_type_path_and_name};
use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{
  parse::{self, Parse, ParseStream, Parser},
  parse_macro_input, parse_quote,
  punctuated::Punctuated,
  DeriveInput, Error, Field, FieldValue, GenericParam, Ident, ImplItem, ItemImpl, ItemStruct,
  Lifetime, LifetimeDef, Path, Token,
};

mod drop_getter_method;
mod drop_method_attribute;
mod helpers;

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

#[proc_macro_attribute]
pub fn lazy_value_view(args: TokenStream, input: TokenStream) -> TokenStream {
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

#[proc_macro_attribute]
pub fn liquid_drop_struct(_args: TokenStream, input: TokenStream) -> TokenStream {
  let mut input = parse_macro_input!(input as ItemStruct);
  let ident = &input.ident;
  let cache_struct_ident = Ident::new(format!("{}Cache", ident).as_str(), Span::call_site().into());

  match &mut input.fields {
    syn::Fields::Named(named_fields) => named_fields.named.push(
      Field::parse_named
        .parse2(quote!(
          drop_cache: #cache_struct_ident<'cache>
        ))
        .unwrap(),
    ),
    _ => unimplemented!(),
  }

  input
    .generics
    .params
    .push(GenericParam::Lifetime(LifetimeDef::new(Lifetime::new(
      "'cache",
      Span::call_site().into(),
    ))));

  quote!(
    #[derive(Debug, Clone)]
    #input
  )
  .into()
}

#[proc_macro_attribute]
pub fn liquid_drop_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as ItemImpl);
  let (self_ty, self_name) = get_type_path_and_name(&input.self_ty).unwrap();
  let type_name = syn::LitStr::new(&self_name, Span::call_site().into());
  let cache_struct_ident = Ident::new(
    format!("{}Cache", self_name).as_str(),
    Span::call_site().into(),
  );

  let (methods, other_items): (Vec<ImplItem>, Vec<ImplItem>) =
    input.items.into_iter().partition(|item| match item {
      syn::ImplItem::Method(method) => method.sig.receiver().is_some(),
      _ => false,
    });
  let (mut constructors, other_items): (Vec<ImplItem>, Vec<ImplItem>) =
    other_items.into_iter().partition(|item| match item {
      ImplItem::Method(method) => {
        method.sig.receiver().is_none()
          && method
            .block
            .stmts
            .last()
            .map(|stmt| match stmt {
              syn::Stmt::Expr(syn::Expr::Struct(struct_expr)) => struct_expr
                .path
                .segments
                .last()
                .map(|last_segment| last_segment.ident == "Self" || last_segment.ident == self_name)
                .unwrap_or(false),
              _ => false,
            })
            .unwrap_or(false)
      }
      _ => false,
    });

  for constructor in &mut constructors {
    match constructor {
      ImplItem::Method(method) => {
        let last_stmt = method.block.stmts.iter_mut().last().unwrap();
        if let syn::Stmt::Expr(syn::Expr::Struct(struct_expr)) = last_stmt {
          let cache_field = FieldValue::parse
            .parse2(quote!(drop_cache: Default::default()))
            .unwrap();

          struct_expr.fields.push(cache_field)
        } else {
          unimplemented!()
        }
      }
      _ => unimplemented!(),
    }
  }
  let methods: Vec<DropGetterMethod> = methods
    .into_iter()
    .filter_map(|method| match method {
      syn::ImplItem::Method(method) => Some(DropGetterMethod::from(method)),
      _ => None,
    })
    .collect();

  let method_getters = methods.iter().map(|method| {
    let getter = method.getter();
    let caching_getter = method.caching_getter();

    quote!(
      #getter
      #caching_getter
    )
  });

  let method_count = methods.len();

  let method_name_strings: Vec<syn::LitStr> = methods
    .iter()
    .map(|getter_method| getter_method.name_str())
    .collect();

  let method_serializers = methods.iter().map(|method| {
    let ident = method.ident();
    let name_str = method.name_str();

    quote!(
      struct_serializer.serialize_field(#name_str, &#ident.to_value())?;
    )
  });

  let cache_fields = methods.iter().map(|method| {
    let ident = method.ident();

    quote!(
      #ident: tokio::sync::OnceCell<::lazy_liquid_value_view::DropResult<'cache>>
    )
  });

  let drop_cache_struct = quote!(
    #[derive(Debug, Clone, Default)]
    struct #cache_struct_ident<'cache> {
      #(#cache_fields),*
    }
  );

  let getter_invocations = methods.iter().map(|method| {
    let caching_getter_ident = method.caching_getter_ident();

    quote!(self.#caching_getter_ident())
  });

  let getter_idents = methods.iter().map(|method| method.ident());

  let destructure_var_names = getter_idents.clone();
  let get_all_blocking = quote!(
    let (#(#destructure_var_names ,)*) = tokio::task::block_in_place(move || {
      tokio::runtime::Handle::current().block_on(async move {
        futures::join!(
          #(#getter_invocations),*
        )
      })
    });
  );

  let serialize_impl = quote!(
    impl<'cache> serde::ser::Serialize for #self_ty<'cache> {
      fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
      where
        S: serde::ser::Serializer,
      {
        use ::serde::ser::SerializeStruct;
        use ::liquid_core::ValueView;

        let mut struct_serializer = serializer.serialize_struct(#type_name, #method_count)?;
        #get_all_blocking
        #(#method_serializers)*
        struct_serializer.end()
      }
    }
  );

  let value_view_impl = quote!(
    impl<'cache> liquid::ValueView for #self_ty<'cache> {
      fn as_debug(&self) -> &dyn std::fmt::Debug {
        self as &dyn std::fmt::Debug
      }

      fn render(&self) -> liquid::model::DisplayCow<'_> {
        liquid::model::DisplayCow::Owned(Box::new(#type_name))
      }

      fn source(&self) -> liquid::model::DisplayCow<'_> {
        liquid::model::DisplayCow::Owned(Box::new(#type_name))
      }

      fn type_name(&self) -> &'static str {
        #type_name
      }

      fn query_state(&self, state: liquid::model::State) -> bool {
        match state {
          liquid::model::State::Truthy => true,
          liquid::model::State::DefaultValue => false,
          liquid::model::State::Empty => false,
          liquid::model::State::Blank => false,
        }
      }

      fn to_kstr(&self) -> liquid::model::KStringCow<'_> {
        #type_name.to_kstr()
      }

      fn to_value(&self) -> liquid_core::Value {
        println!("Warning!  to_value called on {}", #type_name);
        liquid::model::Value::Object(
          liquid::model::Object::from_iter(
            self.as_object().unwrap().iter().map(|(key, value)| (key.into(), value.to_value()))
          )
        )
      }

      fn as_object(&self) -> Option<&dyn ::liquid::model::ObjectView> {
        Some(self)
      }
    }
  );

  let object_pairs = methods.iter().map(|method| {
    let ident = method.ident();
    let name_str = method.name_str();

    quote!(
      (#name_str, #ident)
    )
  });

  let object_getters = methods.iter().map(|method| {
    let ident = method.caching_getter_ident();
    let name_str = method.name_str();

    quote!(
      #name_str => Some(self.#ident().await as &dyn liquid::ValueView)
    )
  });

  let drop_result_from_impl = quote!(
    impl<'a, 'cache: 'a> From<#self_ty<'cache>> for ::lazy_liquid_value_view::DropResult<'a> {
      fn from(drop: #self_ty<'cache>) -> Self {
        ::lazy_liquid_value_view::DropResult::new(drop.clone())
      }
    }

    impl<'a, 'cache: 'a> From<&'a #self_ty<'cache>> for ::lazy_liquid_value_view::DropResult<'a> {
      fn from(drop: &'a #self_ty<'cache>) -> Self {
        ::lazy_liquid_value_view::DropResult::new(drop.clone())
      }
    }
  );

  let object_view_impl = quote!(
    impl<'cache> liquid::ObjectView for #self_ty<'cache> {
      fn as_value(&self) -> &dyn liquid::ValueView {
        self as &dyn liquid::ValueView
      }

      fn size(&self) -> i64 {
        usize::try_into(#method_count).unwrap()
      }

      fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = liquid::model::KStringCow<'k>> + 'k> {
        Box::new(
          vec![
            #(#method_name_strings),*
          ]
          .into_iter()
          .map(|s| s.into()),
        )
      }

      fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn liquid::ValueView> + 'k> {
        #get_all_blocking
        let values = vec![
          #(#getter_idents),*
        ];

        Box::new(values.into_iter().map(|drop_result| drop_result as &dyn ::liquid::ValueView))
      }

      fn iter<'k>(
        &'k self,
      ) -> Box<dyn Iterator<Item = (liquid::model::KStringCow<'k>, &'k dyn liquid::ValueView)> + 'k> {
        #get_all_blocking
        let pairs = vec![
          #(#object_pairs ,)*
        ];

        Box::new(
          pairs
            .into_iter()
            .map(|(key, value)| (key.into(), value as &dyn ::liquid::ValueView)),
        )
      }

      fn contains_key(&self, index: &str) -> bool {
        match index {
          #(#method_name_strings)|* => true,
          _ => false,
        }
      }

      fn get<'s>(&'s self, index: &str) -> Option<&'s dyn liquid::ValueView> {
        tokio::task::block_in_place(move || {
          tokio::runtime::Handle::current().block_on(async move {
            match index {
              #(#object_getters ,)*
              _ => None,
            }
          })
        })
      }
    }
  );

  let ret = quote!(
    #drop_cache_struct

    impl<'cache> #self_ty<'cache> {
      #(#constructors)*
      #(#other_items)*
      #(#method_getters)*

      pub fn extend(&self, extensions: liquid::model::Object) -> ::lazy_liquid_value_view::ExtendedDropResult<'_> {
        ::lazy_liquid_value_view::ExtendedDropResult {
          drop_result: self.into(),
          extensions,
        }
      }
    }

    #serialize_impl
    #value_view_impl
    #object_view_impl
    #drop_result_from_impl
  );

  ret.into()
}
