use quote::{quote, ToTokens};

use super::LiquidDropImpl;

pub fn implement_value_view(liquid_drop_impl: &LiquidDropImpl) -> Box<dyn ToTokens> {
  let generics = &liquid_drop_impl.generics;
  let type_name = &liquid_drop_impl.type_name;
  let self_ty = &liquid_drop_impl.self_ty;

  Box::new(quote!(
    impl #generics liquid::ValueView for #self_ty {
      fn as_debug(&self) -> &dyn std::fmt::Debug {
        self as &dyn std::fmt::Debug
      }

      fn render(&self) -> liquid::model::DisplayCow<'_> {
        liquid::model::DisplayCow::Owned(Box::new(#type_name))
      }

      fn source(&self) -> liquid::model::DisplayCow<'_> {
        liquid::model::DisplayCow::Owned(Box::new(#type_name))
      }

      fn type_name(&self) -> &'static str {
        #type_name
      }

      fn query_state(&self, state: liquid::model::State) -> bool {
        match state {
          liquid::model::State::Truthy => true,
          liquid::model::State::DefaultValue => false,
          liquid::model::State::Empty => false,
          liquid::model::State::Blank => false,
        }
      }

      fn to_kstr(&self) -> liquid::model::KStringCow<'_> {
        #type_name.to_kstr()
      }

      fn to_value(&self) -> liquid_core::Value {
        liquid::model::Value::Object(
          liquid::model::Object::from_iter(
            self.as_object().unwrap().iter().map(|(key, value)| (key.into(), value.to_value()))
          )
        )
      }

      fn as_object(&self) -> Option<&dyn ::liquid::model::ObjectView> {
        Some(self)
      }
    }
  ))
}
