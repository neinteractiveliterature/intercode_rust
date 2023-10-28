use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use serde::Deserialize;
use syn::{
  parse::Parse, parse_macro_input, punctuated::Punctuated, spanned::Spanned, token::Comma, LitStr,
};

const OPERATIONS_JSON: &str = include_str!("../../../graphql_operations_generated.json");

#[derive(Deserialize, Clone)]
struct GraphQLOperationInternal {
  pub document: String,
  pub ast: serde_json::Value,
}

struct LoadOperationsInput {
  pub operation_names: Vec<String>,
}

impl Parse for LoadOperationsInput {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let operation_names = Punctuated::<LitStr, Comma>::parse_terminated(input)?;
    Ok(LoadOperationsInput {
      operation_names: operation_names
        .into_iter()
        .map(|lit_str| lit_str.value())
        .collect(),
    })
  }
}

#[proc_macro]
pub fn load_operations(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as LoadOperationsInput);
  let deserializer = &mut serde_json::Deserializer::from_str(OPERATIONS_JSON);
  let operations_by_name: HashMap<String, GraphQLOperationInternal> =
    serde_path_to_error::deserialize(deserializer).unwrap();

  let operation_additions = input.operation_names.into_iter().map(move |name| {
    let operation = operations_by_name.get(&name).unwrap();
    let name_str = LitStr::new(&name, name.span());
    let document_str = LitStr::new(&operation.document, name.span());
    let ast_json: proc_macro2::TokenStream = format!(
      "::serde_json::json!({})",
      serde_json::to_string(&operation.ast).unwrap()
    )
    .parse()
    .unwrap();

    quote! {
      operations.insert(#name_str.to_string(), GraphQLOperation {
        document: #document_str.to_string(),
        ast: #ast_json
      });
    }
  });

  let capacity = operation_additions.len();

  quote! {
    {
      let mut operations: HashMap<String, GraphQLOperation> = HashMap::with_capacity(#capacity);
      #(#operation_additions)*
      operations
    }
  }
  .into_token_stream()
  .into()
}
