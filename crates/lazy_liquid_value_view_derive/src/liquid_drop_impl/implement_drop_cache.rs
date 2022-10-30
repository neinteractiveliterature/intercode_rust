use quote::{quote, ToTokens};
use syn::{Ident, LitStr, PathArguments};

use super::LiquidDropImpl;

pub fn implement_drop_cache(liquid_drop_impl: &LiquidDropImpl) -> Box<dyn ToTokens> {
  let methods = liquid_drop_impl.methods.iter().collect::<Vec<_>>();
  let self_type_arguments = &liquid_drop_impl.self_type_arguments;
  let cache_struct_ident = &liquid_drop_impl.cache_struct_ident;
  let cache_struct_ident_litstr = LitStr::new(
    cache_struct_ident.to_string().as_str(),
    cache_struct_ident.span(),
  );
  let generics = &liquid_drop_impl.generics;

  let cache_fields = methods.iter().map(|method| {
    let ident = method.cache_field_ident();
    let cache_type = method.cache_type();

    quote!(
      #ident: once_cell::race::OnceBox<::lazy_liquid_value_view::DropResult<#cache_type>>
    )
  });

  let default_fields = methods.iter().map(|method| {
    let ident = method.cache_field_ident();

    quote!(
      #ident: ::once_cell::race::OnceBox::new()
    )
  });

  let cache_field_setters = methods.iter().filter(|method| !method.is_id).map(|method| {
    let ident = method.cache_field_ident();
    let setter_ident = Ident::new(format!("set_{}", ident).as_str(), ident.span());
    let cache_type = method.cache_type();

    quote!(
      pub fn #setter_ident(
        &self,
        value: ::lazy_liquid_value_view::DropResult<#cache_type>,
      ) -> Result<(), Box<::lazy_liquid_value_view::DropResult<#cache_type>>> {
        self.#ident.set(Box::new(value))
      }
    )
  });

  let debug_fields = methods.iter().map(|method| {
    let ident = method.cache_field_ident();
    let field_name = method.name_str();

    quote!(
      field(#field_name, &self.#ident.get())
    )
  });

  let phantom_data = self_type_arguments.as_ref().and_then(|path_args| {
    if let PathArguments::AngleBracketed(angle_bracketed_args) = path_args {
      let args = &angle_bracketed_args.args;
      Some(quote!(_phantom: ::std::marker::PhantomData<(#args)>,))
    } else {
      None
    }
  });

  let phantom_default = phantom_data
    .as_ref()
    .map(|_| quote!(_phantom: Default::default(),));

  Box::new(quote!(
    pub struct #cache_struct_ident #generics {
      #phantom_data
      #(#cache_fields,)*
    }

    impl #generics #cache_struct_ident #self_type_arguments {
      #(#cache_field_setters)*
    }

    impl #generics ::std::fmt::Debug for #cache_struct_ident #self_type_arguments {
      fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_struct(#cache_struct_ident_litstr).#(#debug_fields.)*finish()
      }
    }

    impl #generics Default for #cache_struct_ident #self_type_arguments {
      fn default() -> Self {
        Self {
          #phantom_default
          #(#default_fields,)*
        }
      }
    }
  ))
}