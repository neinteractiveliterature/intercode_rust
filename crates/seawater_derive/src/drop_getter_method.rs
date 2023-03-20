use std::fmt::Debug;

use proc_macro::Span;
use quote::{quote, ToTokens};
use syn::{GenericArgument, Ident, ImplItemMethod, LitStr, Path, Signature, Type, TypeTuple};

use crate::helpers::get_drop_result_generic_arg;

#[derive(Clone, Debug)]
pub struct DropGetterMethod {
  impl_item: DropGetterMethodImplItem,
  pub serialize: bool,
  pub is_id: bool,
}

#[derive(Clone)]
pub enum DropGetterMethodImplItem {
  Base(Box<ImplItemMethod>),
  Async(Box<DropGetterMethodImplItem>),
  Uncached(Box<DropGetterMethodImplItem>),
}

impl Debug for DropGetterMethodImplItem {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Base(_) => f.debug_tuple("Base").finish(),
      Self::Async(arg0) => f.debug_tuple("Async").field(arg0).finish(),
      Self::Uncached(arg0) => f.debug_tuple("Uncached").field(arg0).finish(),
    }
  }
}

impl DropGetterMethodImplItem {
  pub fn sig(&self) -> Signature {
    match self {
      DropGetterMethodImplItem::Base(method) => method.sig.clone(),
      DropGetterMethodImplItem::Uncached(inner_method)
      | DropGetterMethodImplItem::Async(inner_method) => inner_method.sig(),
    }
  }

  pub fn getter(&self) -> &ImplItemMethod {
    match self {
      DropGetterMethodImplItem::Base(method) => method,
      DropGetterMethodImplItem::Uncached(inner_method)
      | DropGetterMethodImplItem::Async(inner_method) => inner_method.getter(),
    }
  }
}

pub fn is_path_to_serializable(path: &Path) -> bool {
  if path.segments.len() != 1 {
    return false;
  }

  let last_segment = path.segments.last().unwrap();

  match last_segment.ident.to_string().as_str() {
    "i64" | "u64" | "usize" | "str" | "String" | "bool" | "DateTime" => true,
    "Vec" | "Option" | "HashMap" => match &last_segment.arguments {
      syn::PathArguments::None => true,
      syn::PathArguments::AngleBracketed(angle_bracketed) => {
        angle_bracketed.args.iter().all(is_serializable_generic_arg)
      }
      syn::PathArguments::Parenthesized(parenthesized) => {
        parenthesized.inputs.iter().all(is_serializable_type)
      }
    },
    "Result" => {
      let output_type = match &last_segment.arguments {
        syn::PathArguments::AngleBracketed(angle_bracketed) => angle_bracketed.args.first(),
        _ => None,
      };

      output_type
        .map(is_serializable_generic_arg)
        .unwrap_or(false)
    }
    _ => false,
  }
}

pub fn is_serializable_generic_arg(arg: &GenericArgument) -> bool {
  match arg {
    GenericArgument::Type(ty) => is_serializable_type(ty),
    _ => false,
  }
}

pub fn is_serializable_type(ty: &Type) -> bool {
  match ty {
    Type::Path(path) => is_path_to_serializable(&path.path),
    Type::Paren(inner_ty) => is_serializable_type(&inner_ty.elem),
    Type::Reference(reference) => is_serializable_type(reference.elem.as_ref()),
    _ => false,
  }
}

impl DropGetterMethod {
  pub fn new(method: ImplItemMethod, serialize: bool, is_id: bool) -> Self {
    DropGetterMethod {
      impl_item: method.into(),
      serialize,
      is_id,
    }
  }

  fn ident(&self) -> Ident {
    self.impl_item.sig().ident
  }

  pub fn cache_field_ident(&self) -> Ident {
    self.ident()
  }

  // For most fields, we rename the original method name to uncached_* and put a caching method at the
  // original name.
  //
  // For the ID field, this is reversed: we keep the original method name the same and put a caching_*
  // method alongside it.  ID methods are expected to conform with LiquidDrop's id method.
  pub fn caching_getter_ident(&self) -> Ident {
    if !self.is_id {
      return self.ident();
    }

    Ident::new(
      format!("caching_{}", self.ident()).as_str(),
      self.ident().span(),
    )
  }

  pub fn uncached_getter_ident(&self) -> Ident {
    if self.is_id {
      return self.ident();
    }

    Ident::new(
      format!("uncached_{}", self.ident()).as_str(),
      self.ident().span(),
    )
  }

  pub fn uncached_getter<'a>(&'a self) -> Box<dyn ToTokens + 'a> {
    let mut method = self.impl_item.getter().clone();
    method.sig.ident = self.uncached_getter_ident();
    Box::new(method)
  }

  pub fn should_serialize(&self) -> bool {
    let return_type = self.return_type();
    if self.serialize {
      return true;
    }

    is_serializable_type(return_type.as_ref())
  }

  pub fn name_str(&self) -> LitStr {
    let ident = self.ident();
    LitStr::new(ident.to_string().as_str(), ident.span())
  }

  pub fn return_type(&self) -> Box<Type> {
    match self.impl_item.sig().output {
      syn::ReturnType::Default => Box::new(Type::Tuple(TypeTuple {
        paren_token: Default::default(),
        elems: Default::default(),
      })),
      syn::ReturnType::Type(_arrow, ty) => ty,
    }
  }

  pub fn cache_type(&self) -> Box<Type> {
    get_drop_result_generic_arg(self.return_type())
  }

  pub fn caching_getter(&self) -> Box<dyn ToTokens> {
    let cache_field_ident = self.cache_field_ident();
    let get_or_init_ident = Ident::new(
      format!("get_or_init_{}", cache_field_ident).as_str(),
      cache_field_ident.span(),
    );
    let caching_getter_ident = self.caching_getter_ident();
    let uncached_getter_ident = self.uncached_getter_ident();
    let return_type = self.cache_type();

    match self.impl_item {
      DropGetterMethodImplItem::Uncached(_) => Box::new(quote!(
        pub async fn #caching_getter_ident(&self) -> ::seawater::DropResult<#return_type> {
          use ::seawater::LiquidDrop;
          self.#uncached_getter_ident().await.into()
        }
      )),
      DropGetterMethodImplItem::Async(_) => Box::new(quote!(
        pub async fn #caching_getter_ident(&self) -> ::seawater::DropResult<#return_type> {
          use ::seawater::{Context, DropStore, LiquidDrop};
          let cache = self.with_drop_store(|store| store.get_drop_cache::<Self>(self.id()));
          cache.#get_or_init_ident(|| {
            Box::<::seawater::DropResult<#return_type>>::new(
              ::tokio::task::block_in_place(|| {
                ::tokio::runtime::Handle::current()
                  .block_on(async move {
                    self.#uncached_getter_ident().await.into()
                  })
                })
              )
            }).clone()
        }
      )),
      _ => Box::new(quote!(
        pub async fn #caching_getter_ident(&self) -> ::seawater::DropResult<#return_type> {
          use ::seawater::{Context, DropStore, LiquidDrop};
          let cache = self.with_drop_store(|store| store.get_drop_cache::<Self>(self.id()));
          cache.#get_or_init_ident(|| {
            Box::new(self.#uncached_getter_ident().into())
          }).clone()
        }
      )),
    }
  }
}

impl From<ImplItemMethod> for DropGetterMethodImplItem {
  fn from(method: ImplItemMethod) -> Self {
    if let Some(uncached_attr) = method.attrs.iter().find(|attr| {
      attr
        .path
        .is_ident(&Ident::new("uncached", Span::call_site().into()))
    }) {
      let mut inner_method = method.clone();
      inner_method.attrs.retain(|attr| attr != uncached_attr);

      return DropGetterMethodImplItem::Uncached(Box::new(DropGetterMethodImplItem::Base(
        Box::new(inner_method),
      )));
    }

    if method.sig.asyncness.is_some() {
      DropGetterMethodImplItem::Async(Box::new(DropGetterMethodImplItem::Base(Box::new(method))))
    } else {
      DropGetterMethodImplItem::Base(Box::new(method))
    }
  }
}
