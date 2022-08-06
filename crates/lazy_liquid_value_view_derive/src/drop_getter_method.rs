use std::fmt::Debug;

use proc_macro::Span;
use quote::{quote, ToTokens};
use syn::{Ident, ImplItemMethod, LitStr, Signature};

#[derive(Clone)]
pub enum DropGetterMethod {
  Base(Box<ImplItemMethod>),
  Async(Box<DropGetterMethod>),
  // Result(Box<DropGetterMethod>, Box<Type>),
  // Option(Box<DropGetterMethod>),
  // SerializeValue(Box<DropGetterMethod>),
  // ValueViewRef(Box<DropGetterMethod>),
}

// fn set_return_type(mut sig: Signature, new_return_type: Box<Type>) -> Signature {
//   if let ReturnType::Type(rarrow, _ret_type) = &sig.output {
//     sig.output = ReturnType::Type(*rarrow, new_return_type)
//   }
//   sig
// }

// fn set_async(mut sig: Signature) -> Signature {
//   sig.asyncness = Some(parse_quote!(async));
//   sig
// }

// fn set_self_lifetime(mut sig: Signature) -> Signature {
//   let receiver = sig.inputs.first_mut();
//   // eprintln!("receiver: {:?}", receiver);
//   let mut reference = match receiver {
//     Some(syn::FnArg::Receiver(receiver)) => receiver.reference.as_mut(),
//     Some(syn::FnArg::Typed(_typed_receiver)) => todo!(),
//     None => {
//       return sig;
//     }
//   };

//   sig.generics.params.push(parse_quote!('value));

//   let mut value_lifetime = (
//     And::default(),
//     Some(Lifetime::new("'value", Span::call_site().into())),
//   );
//   let (_and, lifetime) = reference.get_or_insert(&mut value_lifetime);
//   let _prev = lifetime.insert(Lifetime::new("'value", Span::call_site().into()));
//   sig.generics.where_clause = parse_quote!(where 'cache: 'value);

//   sig
// }

impl Debug for DropGetterMethod {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Base(_) => f.debug_tuple("Base").finish(),
      Self::Async(arg0) => f.debug_tuple("Async").field(arg0).finish(),
      // Self::Result(arg0, arg1) => f.debug_tuple("Result").field(arg0).field(arg1).finish(),
      // Self::Option(arg0) => f.debug_tuple("Option").field(arg0).finish(),
      // Self::SerializeValue(arg0) => f.debug_tuple("SerializeValue").field(arg0).finish(),
      // Self::ValueViewRef(arg0) => f.debug_tuple("ValueViewRef").field(arg0).finish(),
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
      // DropGetterMethod::ValueViewRef(method) => method.sig(),
      DropGetterMethod::Async(inner_method) => inner_method.sig(),
      // DropGetterMethod::Result(inner_method, _) => inner_method.sig(),
      // DropGetterMethod::Option(inner_method) => inner_method.sig(),
      // DropGetterMethod::SerializeValue(inner_method) => inner_method.sig(),
    }
  }

  pub fn caching_getter(&self) -> Box<dyn ToTokens> {
    let ident = self.ident();
    let caching_getter_ident = self.caching_getter_ident();

    match self {
      DropGetterMethod::Async(_) => Box::new(quote!(
        async fn #caching_getter_ident(&self) -> &::lazy_liquid_value_view::DropResult<'_> {
          self
            .drop_cache
            .#ident.
            get_or_init(|| async move {
              self.#ident().await.into()
            })
            .await
        }
      )),
      _ => Box::new(quote!(
        async fn #caching_getter_ident(&self) -> &::lazy_liquid_value_view::DropResult<'_> {
          self
            .drop_cache
            .#ident.
            get_or_init(|| async move {
              self.#ident().into()
            })
            .await
        }
      )),
    }
  }

  pub fn getter<'a>(&'a self) -> Box<dyn ToTokens + 'a> {
    match self {
      DropGetterMethod::Base(method) => Box::new(method),
      DropGetterMethod::Async(inner_method) => inner_method.getter(),
      // DropGetterMethod::ValueViewRef(inner_method) => inner_method.getter(),
      // DropGetterMethod::Result(inner_method, _error_type) => inner_method.getter(),
      // DropGetterMethod::Option(inner_method) => inner_method.getter(),
      // DropGetterMethod::SerializeValue(inner_method) => inner_method.getter(),
    }
  }
}

impl From<ImplItemMethod> for DropGetterMethod {
  fn from(method: ImplItemMethod) -> Self {
    // let drop_attrs: Vec<DropMethodAttribute> =
    //   method.attrs.iter().map(DropMethodAttribute::from).collect();

    // let return_type = extract_return_type(&method.sig.output);

    // if let ReturnType::Type(_, ref boxed_type) = method.sig.output {
    //   if let Type::Reference(type_reference) = boxed_type.as_ref() {
    //     if let Type::TraitObject(type_trait_object) = type_reference.elem.as_ref() {
    //       if type_trait_object.bounds.iter().any(|bound| match bound {
    //         syn::TypeParamBound::Trait(trait_bound) => {
    //           if let Some(last_segment) = trait_bound.path.segments.last() {
    //             last_segment.ident == "ValueView"
    //           } else {
    //             false
    //           }
    //         }
    //         _ => false,
    //       }) {
    //         return DropGetterMethod::ValueViewRef(Box::new(DropGetterMethod::Base(Box::new(
    //           method,
    //         ))));
    //       }
    //     }
    //   }
    // }

    // if drop_attrs
    //   .iter()
    //   .any(|attr| matches!(attr, DropMethodAttribute::SerializeValue))
    // {
    //   let mut inner_method = method;
    //   inner_method.attrs.retain(|attr| {
    //     let parsed_attr = DropMethodAttribute::from(attr);
    //     !matches!(parsed_attr, DropMethodAttribute::SerializeValue)
    //   });
    //   inner_method.sig.output = ReturnType::Type(
    //     Default::default(),
    //     Box::new(parse_quote!(liquid::model::Value)),
    //   );
    //   return DropGetterMethod::SerializeValue(Box::new(inner_method.into()));
    // }

    // if let Some((rarrow, path_only, last_segment_arguments)) = return_type {
    //   match path_only.as_str() {
    //     "i64" | "str" | "String" => {
    //       let mut inner_method = method;
    //       inner_method.sig.output =
    //         ReturnType::Type(rarrow, Box::new(parse_quote!(liquid::model::Value)));
    //       return DropGetterMethod::SerializeValue(Box::new(inner_method.into()));
    //     }
    //     "Result" => {
    //       if let Some(PathArguments::AngleBracketed(args)) = last_segment_arguments {
    //         if let (
    //           Some(GenericArgument::Type(output_type)),
    //           Some(GenericArgument::Type(error_type)),
    //         ) = (args.args.first(), args.args.last())
    //         {
    //           let mut inner_method = method;
    //           inner_method.sig.output = ReturnType::Type(rarrow, Box::new(output_type.to_owned()));
    //           return DropGetterMethod::Result(
    //             Box::new(inner_method.into()),
    //             Box::new(error_type.to_owned()),
    //           );
    //         }
    //       }
    //     }
    //     "Option" => {
    //       if let Some(PathArguments::AngleBracketed(args)) = last_segment_arguments {
    //         if let Some(GenericArgument::Type(output_type)) = args.args.first() {
    //           let mut inner_method = method;
    //           inner_method.sig.output = ReturnType::Type(rarrow, Box::new(output_type.to_owned()));
    //           return DropGetterMethod::Option(Box::new(inner_method.into()));
    //         }
    //       }
    //     }
    //     _ => {}
    //   }
    // }

    if method.sig.asyncness.is_some() {
      DropGetterMethod::Async(Box::new(DropGetterMethod::Base(Box::new(method))))
    } else {
      DropGetterMethod::Base(Box::new(method))
    }
  }
}
