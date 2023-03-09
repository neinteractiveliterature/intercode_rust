use proc_macro::Span;
use quote::{quote, ToTokens};
use syn::{parse_quote, GenericParam, Ident};

use super::LiquidDropImpl;

pub fn implement_drop_result_from(liquid_drop_impl: &LiquidDropImpl) -> Box<dyn ToTokens> {
  let generics = &liquid_drop_impl.generics;
  let self_ty = &liquid_drop_impl.self_ty;
  let self_ty_args = &liquid_drop_impl.self_type_arguments;
  let option_newtype_ident = Ident::new(
    format!("Optional{}", &liquid_drop_impl.self_name).as_str(),
    Span::call_site().into(),
  );

  let mut generics_plus_convertible = generics.clone();
  generics_plus_convertible.params.push(GenericParam::Type(
    parse_quote!(OptionConvertible: Into<#option_newtype_ident #self_ty_args>),
  ));

  Box::new(quote!(
    impl #generics From<#self_ty> for ::seawater::DropResult<#self_ty> {
      fn from(drop: #self_ty) -> Self {
        ::seawater::DropResult::new(drop.clone())
      }
    }

    // struct #option_newtype_ident #generics(Option<#self_ty #self_ty_args>);

    // impl #generics From<Option<#self_ty #self_ty_args>> for #option_newtype_ident #self_ty_args {
    //   fn from(option: Option<#option_newtype_ident #self_ty_args>) -> Self {
    //     Self(option)
    //   }
    // }

    // impl #generics_plus_convertible From<OptionConvertible> for ::seawater::DropResult<#self_ty> {
    //   fn from(option: OptionConvertible) -> Self {
    //     option.into().0.map(|value| value.into()).unwrap_or_default()
    //   }
    // }
  ))
}
