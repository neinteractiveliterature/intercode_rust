use quote::{quote, ToTokens};

use super::LiquidDropImpl;

pub fn implement_drop_result_from(liquid_drop_impl: &LiquidDropImpl) -> Box<dyn ToTokens> {
  let generics = &liquid_drop_impl.generics;
  let self_ty = &liquid_drop_impl.self_ty;

  Box::new(quote!(
    impl #generics From<#self_ty> for ::lazy_liquid_value_view::DropResult<#self_ty> {
      fn from(drop: #self_ty) -> Self {
        ::lazy_liquid_value_view::DropResult::new(drop.clone())
      }
    }
  ))
}
