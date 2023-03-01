use crate::drop_getter_method::DropGetterMethod;
use crate::drop_method_attribute::DropMethodAttribute;
use crate::helpers::get_type_path_and_name_and_arguments;
use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::parse::{self, Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Error, Ident, ImplItem, ItemImpl, Path, PathArguments, Token};
use syn::{Generics, Type};

use self::implement_drop::implement_drop;
use self::implement_drop_cache::implement_drop_cache;
use self::implement_drop_result_from::implement_drop_result_from;
use self::implement_object_view::implement_object_view;
use self::implement_serialize::implement_serialize;
use self::implement_value_view::implement_value_view;

mod implement_drop;
mod implement_drop_cache;
mod implement_drop_result_from;
mod implement_get_all_blocking;
mod implement_object_view;
mod implement_serialize;
mod implement_value_view;

struct LiquidDropImplArgs {
  pub id_type: Option<Path>,
}

impl Parse for LiquidDropImplArgs {
  fn parse(input: ParseStream) -> parse::Result<Self> {
    let vars = Punctuated::<Path, Token![,]>::parse_terminated(input)?;
    let mut vars_iter = vars.iter();
    let id_type = vars_iter.next();

    if vars_iter.next().is_some() {
      return Err(Error::new(
        input.span(),
        "Unexpected parameter for liquid_drop_impl macro",
      ));
    }

    Ok(LiquidDropImplArgs {
      id_type: id_type.map(|path| path.to_owned()),
    })
  }
}

pub struct LiquidDropImpl {
  self_ty: Type,
  self_type_arguments: Option<PathArguments>,
  self_name: String,
  generics: Generics,
  type_name: syn::LitStr,
  cache_struct_ident: Ident,
  constructors: Vec<ImplItem>,
  methods: Vec<DropGetterMethod>,
  other_items: Vec<ImplItem>,
}

impl LiquidDropImpl {
  fn new(input: ItemImpl, has_id_type: bool) -> Self {
    let (self_ty, self_name, self_type_arguments) =
      get_type_path_and_name_and_arguments(&input.self_ty).unwrap();
    let generics = input.generics.clone();
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
    let (constructors, other_items): (Vec<ImplItem>, Vec<ImplItem>) =
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
                  .map(|last_segment| {
                    last_segment.ident == "Self" || last_segment.ident == self_name
                  })
                  .unwrap_or(false),
                _ => false,
              })
              .unwrap_or(false)
        }
        _ => false,
      });

    let mut ignored_methods: Vec<ImplItem> = vec![];

    let getter_methods = methods
      .into_iter()
      .filter_map(|method| match method {
        syn::ImplItem::Method(mut method) => {
          let attrs = method
            .attrs
            .iter()
            .map(DropMethodAttribute::try_from)
            .filter_map(|attr| attr.ok())
            .collect::<Vec<_>>();

          let ignore = attrs
            .iter()
            .any(|attr| matches!(attr, DropMethodAttribute::Ignore));

          let serialize = attrs
            .iter()
            .any(|attr| matches!(attr, DropMethodAttribute::SerializeValue));

          method
            .attrs
            .retain(|attr| DropMethodAttribute::try_from(attr).is_err());

          if ignore {
            ignored_methods.push(syn::ImplItem::Method(method));
            return None;
          }

          let is_id = has_id_type && method.sig.ident == "id";
          Some(DropGetterMethod::new(method, serialize, is_id))
        }
        _ => None,
      })
      .collect::<Vec<_>>();

    LiquidDropImpl {
      self_ty,
      self_type_arguments,
      self_name,
      generics,
      type_name,
      cache_struct_ident,
      constructors,
      methods: getter_methods,
      other_items: other_items
        .into_iter()
        .chain(ignored_methods.into_iter())
        .collect(),
    }
  }
}

pub fn eval_liquid_drop_impl_macro(args: TokenStream, input: TokenStream) -> TokenStream {
  let args = parse_macro_input!(args as LiquidDropImplArgs);
  let input = parse_macro_input!(input as ItemImpl);
  let analyzed_impl = LiquidDropImpl::new(input, args.id_type.is_some());

  let drop_cache_struct = implement_drop_cache(&analyzed_impl);
  let drop_impl = implement_drop(&analyzed_impl, args.id_type.as_ref());
  let serialize_impl = implement_serialize(&analyzed_impl);
  let value_view_impl = implement_value_view(&analyzed_impl);
  let object_view_impl = implement_object_view(&analyzed_impl);
  let drop_result_from_impl = implement_drop_result_from(&analyzed_impl);

  let ret = quote!(
    #drop_cache_struct
    #drop_impl
    #serialize_impl
    #value_view_impl
    #object_view_impl
    #drop_result_from_impl
  );

  ret.into()
}
