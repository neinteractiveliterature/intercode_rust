use quote::{quote, ToTokens};
use syn::Path;

use crate::helpers::build_generic_args;

use super::{implement_get_all_blocking::implement_get_all_blocking, LiquidDropImpl};

pub fn implement_drop(
  liquid_drop_impl: &LiquidDropImpl,
  id_type: &Path,
  context_type: &Path,
) -> Box<dyn ToTokens> {
  let constructors = &liquid_drop_impl.constructors;
  let generics = &liquid_drop_impl.generics;
  let methods = &liquid_drop_impl.methods;
  let self_ty = &liquid_drop_impl.self_ty;
  let other_items = &liquid_drop_impl.other_items;
  let cache_struct_ident = &liquid_drop_impl.cache_struct_ident;
  let where_clause = &generics.where_clause;

  let generic_args = build_generic_args(generics.params.iter());

  let method_getters = methods.iter().map(|method| {
    let caching_getter = method.caching_getter();

    if method.is_id {
      quote!(#caching_getter)
    } else {
      let getter = method.uncached_getter();
      quote!(
        #getter
        #caching_getter
      )
    }
  });

  let id_methods = &liquid_drop_impl
    .methods
    .iter()
    .filter(|method| method.is_id)
    .map(|method| method.uncached_getter())
    .collect::<Vec<_>>();

  let get_all_blocking = implement_get_all_blocking(methods.iter().collect::<Vec<_>>().as_slice());

  Box::new(quote!(
    impl #generics #self_ty #where_clause {
      #(#constructors)*
      #(#other_items)*
      #(#method_getters)*
      #get_all_blocking
    }

    impl #generics ::seawater::LiquidDrop for #self_ty #where_clause {
      type Cache = #cache_struct_ident #generic_args;
      type ID = #id_type;
      type Context = #context_type;

      #(#id_methods)*

      fn get_context(&self) -> &Self::Context {
        &self.context
      }
    }
  ))
}
