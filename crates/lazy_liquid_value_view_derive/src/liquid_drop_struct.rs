use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{parse::Parser, parse_macro_input, Field, Ident, ItemStruct};

use crate::helpers::build_generic_args;

pub fn eval_liquid_drop_struct_macro(_args: TokenStream, input: TokenStream) -> TokenStream {
  let mut input = parse_macro_input!(input as ItemStruct);
  let ident = &input.ident;
  let generic_args = build_generic_args(input.generics.params.iter());
  let cache_struct_ident = Ident::new(format!("{}Cache", ident).as_str(), Span::call_site().into());

  match &mut input.fields {
    syn::Fields::Named(named_fields) => named_fields.named.push(
      Field::parse_named
        .parse2(quote!(
          pub drop_cache: #cache_struct_ident #generic_args
        ))
        .unwrap(),
    ),
    _ => unimplemented!(),
  }

  quote!(
    #[derive(Debug, Clone)]
    #input
  )
  .into()
}
