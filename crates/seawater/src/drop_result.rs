use crate::{optional_value_view::OptionalValueView, DropRef, ExtendedDropResult, LiquidDrop};
use liquid::{ObjectView, ValueView};
use once_cell::race::OnceBox;
use std::{
  fmt::{Debug, Display},
  hash::Hash,
  ops::Deref,
};

// Really I'd like this to be an enum but unfortunately DropRef only works with drops, and we need DropResult to
// be able to wrap more or less anything that implements ValueView
pub trait DropResultTrait<T: ValueView + Clone + ToOwned<Owned = T>>: Send + Sync {
  fn get_inner<'a>(&'a self) -> Box<dyn Deref<Target = T> + 'a>;
}

impl<
    D: LiquidDrop + Clone + Send + Sync,
    StoreID: Eq + Hash + Copy + Send + Sync + Display + Debug,
  > DropResultTrait<D> for DropRef<D, StoreID>
where
  D::ID: Display + Debug,
{
  fn get_inner(&self) -> Box<dyn Deref<Target = D>> {
    Box::new(self.fetch())
  }
}

macro_rules! drop_result_trait_as_ref {
  ($t: ty) => {
    impl DropResultTrait<$t> for $t {
      fn get_inner<'a>(&'a self) -> Box<dyn Deref<Target = $t> + 'a> {
        Box::new(self)
      }
    }
  };
}

drop_result_trait_as_ref!(i32);
drop_result_trait_as_ref!(f32);
drop_result_trait_as_ref!(u32);
drop_result_trait_as_ref!(i64);
drop_result_trait_as_ref!(f64);
drop_result_trait_as_ref!(bool);
drop_result_trait_as_ref!(String);
drop_result_trait_as_ref!(liquid::model::Value);
drop_result_trait_as_ref!(liquid::model::DateTime);

impl<V: ValueView + Send + Sync + Clone + 'static> DropResultTrait<OptionalValueView<V>>
  for OptionalValueView<V>
{
  fn get_inner<'a>(&'a self) -> Box<dyn Deref<Target = OptionalValueView<V>> + 'a> {
    Box::new(self)
  }
}

impl<V: ValueView + Send + Sync + Clone + 'static> DropResultTrait<Vec<V>> for Vec<V> {
  fn get_inner<'a>(&'a self) -> Box<dyn Deref<Target = Vec<V>> + 'a> {
    Box::new(self)
  }
}

pub trait IntoDropResult<V: ValueView + Clone = Self>: Into<DropResult<V>> {}

impl<V: ValueView + Clone + Send + Sync + 'static> IntoDropResult for Vec<V> {}

pub struct DropResult<T: ValueView + Debug + Clone> {
  result: Box<dyn DropResultTrait<T>>,
  owned: OnceBox<T>,
}

impl<T: ValueView + Clone> Debug for DropResult<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("DropResult")
      .field("type", &std::any::type_name::<T>())
      .field("owned", &self.owned)
      .finish_non_exhaustive()
  }
}

impl<T: ValueView + Clone + DropResultTrait<T> + Debug + 'static> Clone for DropResult<T> {
  fn clone(&self) -> Self {
    let cloned = Self::new(self.result.get_inner().clone());
    if let Some(value) = self.owned.get() {
      cloned.owned.set(Box::new(value.clone())).unwrap()
    }

    cloned
  }
}

impl<T: ValueView + Debug + Clone> DropResult<T> {
  pub fn new<R: DropResultTrait<T> + 'static>(result: R) -> Self {
    Self {
      result: Box::new(result),
      owned: OnceBox::new(),
    }
  }

  pub fn get_inner<'a>(&'a self) -> Box<dyn Deref<Target = T> + 'a> {
    self.result.get_inner()
  }

  pub fn extend(&self, extensions: liquid::model::Object) -> ExtendedDropResult<T>
  where
    Self: Sized,
  {
    ExtendedDropResult {
      drop_result: self,
      extensions,
    }
  }

  pub(crate) fn get_value(&self) -> &dyn ValueView {
    self
      .owned
      .get_or_init(|| Box::new(self.result.get_inner().to_owned()))
  }
}

impl<T: ValueView + Clone> liquid::ValueView for DropResult<T> {
  fn as_debug(&self) -> &dyn std::fmt::Debug {
    self
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

impl<T: ValueView + Clone> ObjectView for DropResult<T> {
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

macro_rules! drop_result_self_converter {
  ($t: ty) => {
    impl From<$t> for DropResult<$t> {
      fn from(value: $t) -> Self {
        DropResult::new(value)
      }
    }
  };
}

macro_rules! drop_result_value_converter {
  ($t: ty) => {
    impl From<$t> for DropResult<::liquid::model::Value> {
      fn from(value: $t) -> Self {
        DropResult::new(::liquid::model::Value::scalar(value))
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

drop_result_self_converter!(i32);
drop_result_self_converter!(f32);
drop_result_self_converter!(u32);
drop_result_self_converter!(i64);
drop_result_self_converter!(f64);
drop_result_self_converter!(bool);
drop_result_self_converter!(String);
drop_result_self_converter!(liquid::model::DateTime);

impl From<&str> for DropResult<liquid::model::Value> {
  fn from(string: &str) -> Self {
    DropResult::new(liquid::model::Value::scalar(string.to_owned()))
  }
}

impl From<&str> for DropResult<String> {
  fn from(string: &str) -> Self {
    DropResult::new(string.to_owned())
  }
}

impl From<&serde_json::Value> for DropResult<liquid::model::Value> {
  fn from(value: &serde_json::Value) -> Self {
    DropResult::new(liquid::model::to_value(value).unwrap())
  }
}

impl<
    D: LiquidDrop + Send + Sync,
    StoreID: Eq + Hash + Copy + Send + Sync + Display + Debug + 'static,
  > From<DropRef<D, StoreID>> for DropResult<D>
{
  fn from(value: DropRef<D, StoreID>) -> Self {
    DropResult::new(value)
  }
}

impl<V: ValueView + Clone + Send + Sync + 'static, T: Into<V>> From<Option<T>>
  for DropResult<OptionalValueView<V>>
{
  fn from(option: Option<T>) -> Self {
    DropResult::new(OptionalValueView::from(option.map(|value| value.into())))
  }
}

impl<V: ValueView + Clone + Send + Sync + 'static, T: Into<V>, E> From<Result<T, E>>
  for DropResult<OptionalValueView<V>>
{
  fn from(result: Result<T, E>) -> Self {
    result.ok().into()
  }
}

impl<'a, D: LiquidDrop + Clone, T, E> From<Result<&'a T, E>>
  for &'a DropResult<OptionalValueView<D>>
where
  &'a T: Into<&'a DropResult<D>>,
{
  fn from(result: Result<&'a T, E>) -> Self {
    result.map(|value| value.into()).into()
  }
}

impl<V: ValueView + Clone + Send + Sync + 'static, T: Into<V>> From<Vec<T>> for DropResult<Vec<V>> {
  fn from(values: Vec<T>) -> Self {
    let converted_values = values
      .into_iter()
      .map(|value| value.into())
      .collect::<Vec<V>>();

    DropResult::new(converted_values)
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
