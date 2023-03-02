use crate::{
  optional_value_view::OptionalValueView, DropRef, ExtendedDropResult, LiquidDrop, LiquidDropWithID,
};
use liquid::{ObjectView, ValueView};
use std::fmt::Debug;

pub trait DropResultTrait<T: ValueView>: Debug {
  fn get_inner(&self) -> &T;
  fn get_value(&self) -> Box<&dyn liquid::ValueView>;

  fn extend(&self, extensions: liquid::model::Object) -> ExtendedDropResult<T>
  where
    Self: Sized,
  {
    ExtendedDropResult {
      drop_result: self,
      extensions,
    }
  }
}

impl<'store, D: LiquidDrop + LiquidDropWithID> DropResultTrait<D> for DropRef<'store, D>
where
  D::ID: Debug,
{
  fn get_inner(&self) -> &D {
    self.fetch().as_ref()
  }

  fn get_value(&self) -> Box<&dyn liquid::ValueView> {
    Box::new(self.fetch().as_value())
  }
}

impl<T: ValueView> DropResultTrait<T> for T {
  fn get_inner(&self) -> &T {
    self
  }

  fn get_value(&self) -> Box<&dyn liquid::ValueView> {
    Box::new(self)
  }
}

pub type DropResult<T: ValueView> = Box<dyn DropResultTrait<T>>;

impl<T: ValueView> liquid::ValueView for DropResult<T> {
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

impl<T: ValueView> ObjectView for DropResult<T> {
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

macro_rules! drop_result_value_converter {
  ($t: ty) => {
    impl From<$t> for DropResult<::liquid::model::Value> {
      fn from(value: $t) -> Self {
        Box::new(::liquid::model::Value::scalar(value))
      }
    }
  };
}

drop_result_value_converter!(i32);
drop_result_value_converter!(f32);
drop_result_value_converter!(u32);
drop_result_value_converter!(i64);
drop_result_value_converter!(f64);
drop_result_value_converter!(bool);
drop_result_value_converter!(String);
drop_result_value_converter!(liquid::model::DateTime);

impl From<&str> for DropResult<String> {
  fn from(string: &str) -> Self {
    Box::new(string.to_owned())
  }
}

impl From<&serde_json::Value> for DropResult<liquid::model::Value> {
  fn from(value: &serde_json::Value) -> Self {
    Box::new(liquid::model::to_value(value).unwrap())
  }
}

impl<V: ValueView, T: Into<V>> From<Option<T>> for DropResult<OptionalValueView<V>> {
  fn from(option: Option<T>) -> Self {
    Box::new(OptionalValueView::from(option.map(|value| value.into())))
  }
}

impl<V: ValueView, T: Into<V>, E> From<Result<T, E>> for DropResult<OptionalValueView<V>> {
  fn from(result: Result<T, E>) -> Self {
    result.ok().into()
  }
}

impl<'a, D: LiquidDrop + LiquidDropWithID, T, E> From<Result<&'a T, E>>
  for &'a DropResult<OptionalValueView<D>>
where
  &'a T: Into<&'a DropResult<D>>,
{
  fn from(result: Result<&'a T, E>) -> Self {
    result.map(|value| value.into()).into()
  }
}

impl<V: ValueView, T: Into<V>> From<Vec<T>> for DropResult<Vec<V>> {
  fn from(values: Vec<T>) -> Self {
    let converted_values = values
      .into_iter()
      .map(|value| value.into())
      .collect::<Vec<V>>();

    Box::new(converted_values)
  }
}

// impl<T: ValueView + Clone> IntoIterator for DropResult<Vec<T>> {
//   type Item = T;
//   type IntoIter = IntoIter<T>;

//   fn into_iter(self) -> Self::IntoIter {
//     let inner_vec: &Vec<T> = self.get_inner();
//     self.get_inner().iter().cloned()
//   }
// }
