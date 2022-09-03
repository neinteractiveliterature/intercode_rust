use quote::{quote, ToTokens};
use syn::{
  parse::{Parse, Parser},
  FieldValue, ImplItem,
};

use crate::helpers::build_generic_args;

use super::LiquidDropImpl;

pub fn implement_drop(liquid_drop_impl: &LiquidDropImpl) -> Box<dyn ToTokens> {
  let mut constructors = liquid_drop_impl.constructors.clone();
  let generics = &liquid_drop_impl.generics;
  let methods = &liquid_drop_impl.methods;
  let self_ty = &liquid_drop_impl.self_ty;
  let other_items = &liquid_drop_impl.other_items;
  let cache_struct_ident = &liquid_drop_impl.cache_struct_ident;

  add_drop_cache_to_constructors(&mut constructors);
  let generic_args = build_generic_args(generics.params.iter());

  let method_getters = methods.iter().map(|method| {
    let getter = method.getter();
    let caching_getter = method.caching_getter();

    quote!(
      #getter
      #caching_getter
    )
  });

  Box::new(quote!(
    impl #generics #self_ty {
      #(#constructors)*
      #(#other_items)*
      #(#method_getters)*

      pub fn extend(&self, extensions: liquid::model::Object) -> ::lazy_liquid_value_view::ExtendedDropResult<#self_ty> {
        ::lazy_liquid_value_view::ExtendedDropResult {
          drop_result: self.into(),
          extensions,
        }
      }
    }

    impl #generics ::lazy_liquid_value_view::LiquidDrop for #self_ty {
      type Cache = #cache_struct_ident #generic_args;

      fn get_cache(&self) -> &#cache_struct_ident #generic_args {
        &self.drop_cache
      }
    }
  ))
}

fn add_drop_cache_to_constructors(constructors: &mut Vec<ImplItem>) {
  for constructor in constructors {
    match constructor {
      ImplItem::Method(method) => {
        let last_stmt = method.block.stmts.iter_mut().last().unwrap();
        if let syn::Stmt::Expr(syn::Expr::Struct(struct_expr)) = last_stmt {
          let cache_field = FieldValue::parse
            .parse2(quote!(drop_cache: Default::default()))
            .unwrap();

          struct_expr.fields.push(cache_field)
        } else {
          unimplemented!()
        }
      }
      _ => unimplemented!(),
    }
  }
}
