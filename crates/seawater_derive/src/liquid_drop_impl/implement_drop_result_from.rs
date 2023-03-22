use quote::{quote, ToTokens};

use super::LiquidDropImpl;

pub fn implement_drop_result_from(liquid_drop_impl: &LiquidDropImpl) -> Box<dyn ToTokens> {
  let generics = &liquid_drop_impl.generics;
  let self_ty = &liquid_drop_impl.self_ty;
  let where_clause = &generics.where_clause;

  Box::new(quote!(
    impl #generics From<#self_ty> for ::seawater::DropResult<#self_ty> #where_clause {
      fn from(drop: #self_ty) -> Self {
        ::seawater::DropResult::new(drop.clone())
      }
    }

    impl #generics ::seawater::IntoDropResult for #self_ty #where_clause {}

    impl #generics ::seawater::DropResultTrait<#self_ty> for #self_ty #where_clause {
      fn get_inner<'a>(&'a self) -> Option<Box<dyn ::std::ops::Deref<Target = #self_ty> + 'a>> {
        Some(Box::new(self))
      }
    }
  ))
}
