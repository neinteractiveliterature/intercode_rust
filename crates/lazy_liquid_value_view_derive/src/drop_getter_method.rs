use std::fmt::Debug;

use proc_macro::Span;
use quote::{quote, ToTokens};
use syn::{Ident, ImplItemMethod, LitStr, Signature, Type, TypeTuple};

use crate::helpers::get_drop_result_generic_arg;

#[derive(Clone)]
pub enum DropGetterMethod {
  Base(Box<ImplItemMethod>),
  Async(Box<DropGetterMethod>),
  Uncached(Box<DropGetterMethod>),
}

impl Debug for DropGetterMethod {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Base(_) => f.debug_tuple("Base").finish(),
      Self::Async(arg0) => f.debug_tuple("Async").field(arg0).finish(),
      Self::Uncached(arg0) => f.debug_tuple("Uncached").field(arg0).finish(),
    }
  }
}

impl DropGetterMethod {
  pub fn ident(&self) -> Ident {
    self.sig().ident
  }

  pub fn caching_getter_ident(&self) -> Ident {
    Ident::new(
      format!("caching_{}", self.ident()).as_str(),
      Span::call_site().into(),
    )
  }

  pub fn name_str(&self) -> LitStr {
    let ident = self.ident();
    LitStr::new(ident.to_string().as_str(), ident.span())
  }

  pub fn sig(&self) -> Signature {
    match self {
      DropGetterMethod::Base(method) => method.sig.clone(),
      DropGetterMethod::Uncached(inner_method) | DropGetterMethod::Async(inner_method) => {
        inner_method.sig()
      }
    }
  }

  pub fn return_type(&self) -> Box<Type> {
    match self.sig().output {
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
    let ident = self.ident();
    let caching_getter_ident = self.caching_getter_ident();
    let return_type = self.cache_type();

    let caching_getter_sig = quote!(async fn #caching_getter_ident(&self) -> &::lazy_liquid_value_view::DropResult<#return_type>);

    match self {
      DropGetterMethod::Uncached(_) => Box::new(quote!(
        #caching_getter_sig {
          use ::lazy_liquid_value_view::LiquidDropWithID;
          self.#ident().await.into()
        }
      )),
      DropGetterMethod::Async(_) => Box::new(quote!(
        #caching_getter_sig {
          use ::lazy_liquid_value_view::LiquidDropWithID;
          self
            .drop_cache
            .#ident.
            get_or_init(
              || Box::<::lazy_liquid_value_view::DropResult<#return_type>>::new(
                ::tokio::task::block_in_place(|| {
                  ::tokio::runtime::Handle::current()
                    .block_on(async move {
                      self.#ident().await.into()
                    })
                  })
                )
              )
        }
      )),
      _ => Box::new(quote!(
        #caching_getter_sig {
          use ::lazy_liquid_value_view::LiquidDropWithID;
          self
            .drop_cache
            .#ident.
            get_or_init(|| {
              Box::new(self.#ident().into())
            })
        }
      )),
    }
  }

  pub fn getter<'a>(&'a self) -> Box<dyn ToTokens + 'a> {
    match self {
      DropGetterMethod::Base(method) => Box::new(method),
      DropGetterMethod::Uncached(inner_method) | DropGetterMethod::Async(inner_method) => {
        inner_method.getter()
      }
    }
  }
}

impl From<ImplItemMethod> for DropGetterMethod {
  fn from(method: ImplItemMethod) -> Self {
    if let Some(uncached_attr) = method.attrs.iter().find(|attr| {
      attr
        .path
        .is_ident(&Ident::new("uncached", Span::call_site().into()))
    }) {
      let mut inner_method = method.clone();
      inner_method.attrs.retain(|attr| attr != uncached_attr);

      return DropGetterMethod::Uncached(Box::new(DropGetterMethod::Base(Box::new(inner_method))));
    }

    if method.sig.asyncness.is_some() {
      DropGetterMethod::Async(Box::new(DropGetterMethod::Base(Box::new(method))))
    } else {
      DropGetterMethod::Base(Box::new(method))
    }
  }
}
