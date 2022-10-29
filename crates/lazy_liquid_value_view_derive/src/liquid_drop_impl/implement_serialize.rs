use quote::{quote, ToTokens};

use super::{implement_get_all_blocking::implement_get_all_blocking, LiquidDropImpl};

pub fn implement_serialize(liquid_drop_impl: &LiquidDropImpl) -> Box<dyn ToTokens> {
  let generics = &liquid_drop_impl.generics;
  let self_ty = &liquid_drop_impl.self_ty;
  let type_name = &liquid_drop_impl.type_name;
  let method_count = liquid_drop_impl.methods.len();
  let methods = liquid_drop_impl.methods.iter().collect::<Vec<_>>();

  let get_all_blocking = implement_get_all_blocking(&methods);

  let method_serializers = methods.iter().map(|method| {
    let ident = method.caching_getter_ident();
    let name_str = method.name_str();

    quote!(
      struct_serializer.serialize_field(#name_str, &#ident.to_value())?;
    )
  });

  Box::new(quote!(
    impl #generics serde::ser::Serialize for #self_ty {
      fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
      where
        S: serde::ser::Serializer,
      {
        use ::serde::ser::SerializeStruct;
        use ::liquid_core::ValueView;
        use ::lazy_liquid_value_view::LiquidDropWithID;

        let mut struct_serializer = serializer.serialize_struct(#type_name, #method_count)?;
        #get_all_blocking
        #(#method_serializers)*
        struct_serializer.end()
      }
    }
  ))
}
