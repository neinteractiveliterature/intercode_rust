use std::fmt::Debug;

use quote::{quote, ToTokens};
use syn::{
  parse_quote, GenericArgument, Ident, ImplItemMethod, LitStr, PathArguments, ReturnType,
  Signature, Type,
};

use crate::{drop_method_attribute::DropMethodAttribute, helpers::extract_return_type};

#[derive(Clone)]
pub enum DropGetterMethod {
  Base(Box<ImplItemMethod>),
  Async(Box<DropGetterMethod>),
  Result(Box<DropGetterMethod>, Box<Type>),
  Option(Box<DropGetterMethod>),
  SerializeValue(Box<DropGetterMethod>),
}

fn set_return_type(mut sig: Signature, new_return_type: Box<Type>) -> Signature {
  if let ReturnType::Type(rarrow, _ret_type) = &sig.output {
    sig.output = ReturnType::Type(*rarrow, new_return_type)
  }
  sig
}

fn set_async(mut sig: Signature) -> Signature {
  sig.asyncness = Some(parse_quote!(async));
  sig
}

impl Debug for DropGetterMethod {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Base(_) => f.debug_tuple("Base").finish(),
      Self::Async(arg0) => f.debug_tuple("Async").field(arg0).finish(),
      Self::Result(arg0, arg1) => f.debug_tuple("Result").field(arg0).field(arg1).finish(),
      Self::Option(arg0) => f.debug_tuple("Option").field(arg0).finish(),
      Self::SerializeValue(arg0) => f.debug_tuple("SerializeValue").field(arg0).finish(),
    }
  }
}

impl DropGetterMethod {
  pub fn ident(&self) -> Ident {
    self.sig().ident
  }

  pub fn name_str(&self) -> LitStr {
    let ident = self.ident();
    LitStr::new(ident.to_string().as_str(), ident.span())
  }

  pub fn sig(&self) -> Signature {
    set_async(match self {
      DropGetterMethod::Base(method) => method.sig.clone(),
      DropGetterMethod::Async(inner_method) => inner_method.sig(),
      DropGetterMethod::Result(inner_method, _) => set_return_type(
        inner_method.sig(),
        Box::new(parse_quote!(&dyn liquid::ValueView)),
      ),
      DropGetterMethod::Option(inner_method) => set_return_type(
        inner_method.sig(),
        Box::new(parse_quote!(&dyn liquid::ValueView)),
      ),
      DropGetterMethod::SerializeValue(inner_method) => set_return_type(
        inner_method.sig(),
        Box::new(parse_quote!(&dyn liquid::ValueView)),
      ),
    })
  }

  pub fn getter(&self) -> Box<dyn ToTokens> {
    match self {
      DropGetterMethod::Base(method) => {
        let stmts = &method.block.stmts;

        Box::new(quote!(#(#stmts)*))
      }
      DropGetterMethod::Async(inner_method) => inner_method.getter(),
      DropGetterMethod::Result(inner_method, _error_type) => {
        let inner_getter = inner_method.getter();

        Box::new(quote!(
          #inner_getter.unwrap_or(&liquid::model::Value::Nil) as &dyn liquid::ValueView
        ))
      }
      DropGetterMethod::Option(inner_method) => {
        if let DropGetterMethod::SerializeValue(_) = inner_method.as_ref() {
          return inner_method.getter();
        }

        let inner_getter = inner_method.getter();

        Box::new(quote!(
          #inner_getter.unwrap_or(&liquid::model::Value::Nil) as &dyn liquid::ValueView
        ))
      }
      DropGetterMethod::SerializeValue(inner_method) => {
        let ident = self.ident();

        if let DropGetterMethod::Option(inner_inner_method) = inner_method.as_ref() {
          let inner_inner_getter = inner_inner_method.getter();

          Box::new(quote!(
            self
              .drop_cache
              .#ident.
              get_or_init(|| async move {
                #inner_inner_getter
                  .and_then(|value| liquid::model::to_value(&value).ok())
                  .unwrap_or(liquid::model::Value::Nil)
              })
              .await
              .as_view()
          ))
        } else if let DropGetterMethod::Result(inner_inner_method, error_type) =
          inner_method.as_ref()
        {
          let inner_inner_getter = inner_inner_method.getter();

          Box::new(quote!(
            self
              .drop_cache
              .#ident
              .get_or_try_init(|| async move {
                #inner_inner_getter
                .and_then(|value| liquid::model::to_value(&value).map_err(|error| #error_type::from(error)))
              })
              .await
              .unwrap_or(&liquid::model::Value::Nil)
              .as_view()
          ))
        } else {
          let inner_getter = inner_method.getter();

          Box::new(quote!(
            self
              .drop_cache
              .#ident
              .get_or_init(|| async move {
                liquid::model::to_value(&#inner_getter).unwrap_or(liquid::model::Value::Nil)
              })
              .await
              .as_view()
          ))
        }
      }
    }
  }
}

impl From<ImplItemMethod> for DropGetterMethod {
  fn from(method: ImplItemMethod) -> Self {
    let drop_attrs: Vec<DropMethodAttribute> =
      method.attrs.iter().map(DropMethodAttribute::from).collect();

    if drop_attrs
      .iter()
      .any(|attr| matches!(attr, DropMethodAttribute::SerializeValue))
    {
      let mut inner_method = method;
      inner_method.attrs.retain(|attr| {
        let parsed_attr = DropMethodAttribute::from(attr);
        !matches!(parsed_attr, DropMethodAttribute::SerializeValue)
      });
      return DropGetterMethod::SerializeValue(Box::new(inner_method.into()));
    }

    let return_type = extract_return_type(&method.sig.output);

    if let Some((rarrow, path_only, last_segment_arguments)) = return_type {
      match path_only.as_str() {
        "i64" | "str" | "String" => {
          let mut inner_method = method;
          inner_method.sig.output =
            ReturnType::Type(rarrow, Box::new(parse_quote!(liquid::model::Value)));
          return DropGetterMethod::SerializeValue(Box::new(inner_method.into()));
        }
        "Result" => {
          if let Some(PathArguments::AngleBracketed(args)) = last_segment_arguments {
            if let (
              Some(GenericArgument::Type(output_type)),
              Some(GenericArgument::Type(error_type)),
            ) = (args.args.first(), args.args.last())
            {
              let mut inner_method = method;
              inner_method.sig.output = ReturnType::Type(rarrow, Box::new(output_type.to_owned()));
              return DropGetterMethod::Result(
                Box::new(inner_method.into()),
                Box::new(error_type.to_owned()),
              );
            }
          }
        }
        "Option" => {
          if let Some(PathArguments::AngleBracketed(args)) = last_segment_arguments {
            if let Some(GenericArgument::Type(output_type)) = args.args.first() {
              let mut inner_method = method;
              inner_method.sig.output = ReturnType::Type(rarrow, Box::new(output_type.to_owned()));
              return DropGetterMethod::Option(Box::new(inner_method.into()));
            }
          }
        }
        _ => {}
      }
    }

    if method.sig.asyncness.is_some() {
      let mut inner_method = method;
      inner_method.sig.asyncness = None;
      return DropGetterMethod::Async(Box::new(inner_method.into()));
    }

    DropGetterMethod::Base(Box::new(method))
  }
}
