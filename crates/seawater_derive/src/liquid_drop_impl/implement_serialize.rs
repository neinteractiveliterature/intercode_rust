use quote::{quote, ToTokens};

use super::LiquidDropImpl;

pub fn implement_serialize(liquid_drop_impl: &LiquidDropImpl) -> Box<dyn ToTokens> {
  let generics = &liquid_drop_impl.generics;
  let self_ty = &liquid_drop_impl.self_ty;
  let type_name = &liquid_drop_impl.type_name;
  let method_count = liquid_drop_impl.methods.len();
  let methods = liquid_drop_impl.methods.iter().collect::<Vec<_>>();
  let where_clause = &generics.where_clause;

  let method_serializers = methods.iter().map(|method| {
    let name_str = method.name_str();

    quote!(
      struct_serializer.serialize_field(#name_str, &index_map.get(#name_str).unwrap().as_ref().to_value())?;
    )
  });

  Box::new(quote!(
    impl #generics serde::ser::Serialize for #self_ty #where_clause {
      fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
      where
        S: serde::ser::Serializer,
      {
        use ::serde::ser::SerializeStruct;
        use ::liquid_core::ValueView;
        use ::seawater::LiquidDrop;

        let mut struct_serializer = serializer.serialize_struct(#type_name, #method_count)?;
        let index_map = self.get_all_blocking();
        #(#method_serializers)*
        struct_serializer.end()
      }
    }
  ))
}
