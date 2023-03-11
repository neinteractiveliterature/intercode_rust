use crate::{optional_value_view::OptionalValueView, DropError, DropResult};
use liquid::ValueView;
use std::{
  collections::{hash_map::IterMut, HashMap},
  hash::Hash,
};

#[derive(Debug)]
pub struct PreloaderResult<Id: Eq + Hash, Value: ValueView + Clone + Send + Sync + 'static> {
  values_by_id: HashMap<Id, DropResult<Value>>,
}

impl<Id: Eq + Hash, Value: ValueView + Clone + Send + Sync + 'static> PreloaderResult<Id, Value> {
  pub fn new(values_by_id: HashMap<Id, DropResult<Value>>) -> Self {
    Self { values_by_id }
  }

  pub fn get(&self, id: Id) -> DropResult<OptionalValueView<Value>> {
    DropResult::new::<OptionalValueView<Value>>(
      self
        .values_by_id
        .get(&id)
        .map(|drop_result| drop_result.get_inner())
        .into(),
    )
  }

  #[allow(dead_code)]
  pub fn expect_value(&self, id: Id) -> Result<Value, DropError> {
    self
      .get(id)
      .get_inner()
      .as_option()
      .ok_or_else(|| DropError::ExpectedEntityNotFound(std::any::type_name::<Value>().to_string()))
      .cloned()
  }

  pub fn all_values(&self) -> Box<dyn Iterator<Item = &DropResult<Value>> + '_> {
    Box::new(self.values_by_id.values())
  }

  pub fn iter_mut(&mut self) -> IterMut<Id, DropResult<Value>> {
    self.values_by_id.iter_mut()
  }

  // pub fn all_values_unwrapped<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Value> + 'a> {
  //   Box::new(self.all_values().into_iter().map(|value| value.get_value()))
  // }

  pub fn extend(&mut self, items: &mut dyn Iterator<Item = (Id, DropResult<Value>)>) {
    self.values_by_id.extend(items)
  }
}

// impl<Id: Eq + Hash, InnerValue: ValueView + Clone + Send + Sync>
//   PreloaderResult<Id, Vec<InnerValue>>
// {
//   pub fn all_values_flat<'a>(&'a self) -> Box<dyn Iterator<Item = &'a InnerValue> + 'a> {
//     Box::new(self.all_values().flat_map(|v| v.get_inner().as_ref()))
//   }
// }

// impl<Id: Eq + Hash, InnerValue: ValueView + Clone + Send + Sync>
//   PreloaderResult<Id, Vec<ArcValueView<InnerValue>>>
// {
//   pub fn all_values_flat_unwrapped<'a>(&'a self) -> Box<dyn Iterator<Item = &'a InnerValue> + 'a> {
//     Box::new(self.all_values_flat().map(|v| v.as_ref()))
//   }
// }
