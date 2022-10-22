use crate::{ArcValueView, ExtendedDropResult};
use liquid::ObjectView;
use std::sync::Arc;

#[derive(Debug)]
pub struct DropResult<T: liquid::ValueView> {
  pub value: Option<ArcValueView<T>>,
}

impl<T: liquid::model::ValueView> Clone for DropResult<T> {
  fn clone(&self) -> Self {
    Self {
      value: self.value.clone(),
    }
  }
}

impl<T: liquid::model::ValueView> Default for DropResult<T> {
  fn default() -> Self {
    Self { value: None }
  }
}

impl<T: liquid::model::ValueView> DropResult<T> {
  pub fn empty<'a>() -> &'a DropResult<T> {
    &DropResult { value: None }
  }

  pub fn extend(&self, extensions: liquid::model::Object) -> ExtendedDropResult<T> {
    ExtendedDropResult {
      drop_result: self.clone(),
      extensions,
    }
  }

  pub fn get_inner(&self) -> Option<&T> {
    self.value.as_deref()
  }

  pub fn get_shared(&self) -> Option<ArcValueView<T>> {
    self.value.clone()
  }

  pub fn get_value(&self) -> Box<&dyn liquid::ValueView> {
    match &self.value {
      Some(value) => Box::new(value.as_ref()),
      None => Box::new(&liquid::model::Value::Nil),
    }
  }
}

impl<T: liquid::model::ValueView> liquid::ValueView for DropResult<T> {
  fn as_debug(&self) -> &dyn std::fmt::Debug {
    self.get_value().as_debug()
  }

  fn as_object(&self) -> Option<&dyn liquid::ObjectView> {
    self.get_value().as_object()
  }

  fn as_array(&self) -> Option<&dyn liquid::model::ArrayView> {
    self.get_value().as_array()
  }

  fn as_scalar(&self) -> Option<liquid::model::ScalarCow<'_>> {
    self.get_value().as_scalar()
  }

  fn as_state(&self) -> Option<liquid::model::State> {
    self.get_value().as_state()
  }

  fn is_array(&self) -> bool {
    self.get_value().is_array()
  }

  fn is_object(&self) -> bool {
    self.get_value().is_object()
  }

  fn is_scalar(&self) -> bool {
    self.get_value().is_scalar()
  }

  fn is_state(&self) -> bool {
    self.get_value().is_state()
  }

  fn is_nil(&self) -> bool {
    self.get_value().is_nil()
  }

  fn render(&self) -> liquid::model::DisplayCow<'_> {
    self.get_value().render()
  }

  fn source(&self) -> liquid::model::DisplayCow<'_> {
    self.get_value().source()
  }

  fn type_name(&self) -> &'static str {
    self.get_value().type_name()
  }

  fn query_state(&self, state: liquid::model::State) -> bool {
    self.get_value().query_state(state)
  }

  fn to_kstr(&self) -> liquid::model::KStringCow<'_> {
    self.get_value().to_kstr()
  }

  fn to_value(&self) -> liquid_core::Value {
    self.get_value().to_value()
  }
}

impl<T: liquid::model::ValueView> ObjectView for DropResult<T> {
  fn as_value(&self) -> &dyn liquid::ValueView {
    self
  }

  fn size(&self) -> i64 {
    self.get_value().as_object().unwrap().size()
  }

  fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = liquid::model::KStringCow<'k>> + 'k> {
    self.get_value().as_object().unwrap().keys()
  }

  fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn liquid::ValueView> + 'k> {
    self.get_value().as_object().unwrap().values()
  }

  fn iter<'k>(
    &'k self,
  ) -> Box<dyn Iterator<Item = (liquid::model::KStringCow<'k>, &'k dyn liquid::ValueView)> + 'k> {
    self.get_value().as_object().unwrap().iter()
  }

  fn contains_key(&self, index: &str) -> bool {
    self.get_value().as_object().unwrap().contains_key(index)
  }

  fn get<'s>(&'s self, index: &str) -> Option<&'s dyn liquid::ValueView> {
    self.get_value().as_object().unwrap().get(index)
  }
}

impl<T: liquid::model::ValueView> DropResult<T> {
  pub fn new(value_view: T) -> Self {
    DropResult {
      value: Some(ArcValueView(Arc::new(value_view))),
    }
  }
}

macro_rules! drop_result_value_converter {
  ($t: ty) => {
    impl From<$t> for DropResult<$t> {
      fn from(value: $t) -> Self {
        DropResult::new(value)
      }
    }
  };
}

drop_result_value_converter!(i64);
drop_result_value_converter!(f64);
drop_result_value_converter!(bool);
drop_result_value_converter!(String);
drop_result_value_converter!(liquid::model::DateTime);
drop_result_value_converter!(liquid::model::Value);

impl From<&str> for DropResult<String> {
  fn from(string: &str) -> Self {
    DropResult::new(string.to_owned())
  }
}

impl<T: liquid::model::ValueView + Clone> From<&T> for DropResult<T> {
  fn from(value_view_ref: &T) -> Self {
    DropResult::new(value_view_ref.to_owned())
  }
}

impl<T: liquid::model::ValueView> From<Arc<T>> for DropResult<ArcValueView<T>> {
  fn from(value_view_arc: Arc<T>) -> Self {
    DropResult::new(ArcValueView(value_view_arc))
  }
}

impl<T: liquid::model::ValueView> From<ArcValueView<T>> for DropResult<ArcValueView<T>> {
  fn from(arc_value_view: ArcValueView<T>) -> Self {
    DropResult::new(arc_value_view)
  }
}

impl From<&serde_json::Value> for DropResult<liquid::model::Value> {
  fn from(value: &serde_json::Value) -> Self {
    DropResult::new(liquid::model::to_value(value).unwrap())
  }
}

impl<V: liquid::ValueView> From<Arc<V>> for DropResult<V> {
  fn from(arc: Arc<V>) -> Self {
    DropResult {
      value: Some(ArcValueView(arc)),
    }
  }
}

impl<V: liquid::ValueView, T: Into<DropResult<V>>> From<Option<T>> for DropResult<V> {
  fn from(option: Option<T>) -> Self {
    option.map(|value| value.into()).unwrap_or_default()
  }
}

impl<V: liquid::ValueView, T: Into<DropResult<V>>, E> From<Result<T, E>> for DropResult<V> {
  fn from(result: Result<T, E>) -> Self {
    result.map(|value| value.into()).unwrap_or_default()
  }
}

impl<'a, V: liquid::ValueView, T, E> From<Result<&'a T, E>> for &'a DropResult<V>
where
  &'a T: Into<&'a DropResult<V>>,
{
  fn from(result: Result<&'a T, E>) -> Self {
    result
      .map(|value| value.into())
      .unwrap_or_else(|_| DropResult::empty())
  }
}

impl<V: liquid::ValueView, T: Into<V>> From<Vec<T>> for DropResult<Vec<V>> {
  fn from(values: Vec<T>) -> Self {
    let converted_values: Vec<V> = values
      .into_iter()
      .map(|value| value.into())
      .collect::<Vec<V>>();

    DropResult::new(converted_values)
  }
}

impl<'a, T: liquid::ValueView + Clone> From<&'a DropResult<T>> for DropResult<T> {
  fn from(result: &'a DropResult<T>) -> Self {
    result.to_owned()
  }
}
