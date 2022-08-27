use std::{ops::Deref, sync::Arc};

#[derive(Debug)]
pub struct ArcValueView<T: liquid::model::ValueView>(pub Arc<T>);

impl<T: liquid::model::ValueView> Clone for ArcValueView<T> {
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}

impl<T: liquid::model::ValueView> AsRef<T> for ArcValueView<T> {
  fn as_ref(&self) -> &T {
    self.0.as_ref()
  }
}

impl<T: liquid::model::ValueView> Deref for ArcValueView<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    self.0.as_ref()
  }
}

impl<T: liquid::model::ValueView> liquid::model::ValueView for ArcValueView<T> {
  fn as_scalar(&self) -> Option<liquid::model::ScalarCow<'_>> {
    self.0.as_scalar()
  }

  fn is_scalar(&self) -> bool {
    self.0.is_scalar()
  }

  fn as_array(&self) -> Option<&dyn liquid::model::ArrayView> {
    self.0.as_array()
  }

  fn is_array(&self) -> bool {
    self.0.is_array()
  }

  fn as_object(&self) -> Option<&dyn liquid::ObjectView> {
    self.0.as_object()
  }

  fn is_object(&self) -> bool {
    self.0.is_object()
  }

  fn as_state(&self) -> Option<liquid::model::State> {
    self.0.as_state()
  }

  fn is_state(&self) -> bool {
    self.0.is_state()
  }

  fn is_nil(&self) -> bool {
    self.0.is_nil()
  }

  fn as_debug(&self) -> &dyn std::fmt::Debug {
    self.0.as_debug()
  }

  fn render(&self) -> liquid::model::DisplayCow<'_> {
    self.0.render()
  }

  fn source(&self) -> liquid::model::DisplayCow<'_> {
    self.0.source()
  }

  fn type_name(&self) -> &'static str {
    self.0.type_name()
  }

  fn query_state(&self, state: liquid::model::State) -> bool {
    self.0.query_state(state)
  }

  fn to_kstr(&self) -> liquid::model::KStringCow<'_> {
    self.0.to_kstr()
  }

  fn to_value(&self) -> liquid_core::Value {
    self.0.to_value()
  }
}
