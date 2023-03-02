use crate::DropError;
use crate::{ArcValueView, DropResult};
use liquid::ValueView;
use std::{
  collections::{hash_map::IterMut, HashMap},
  hash::Hash,
};

#[derive(Debug)]
pub struct PreloaderResult<Id: Eq + Hash, Value: ValueView + Clone> {
  values_by_id: HashMap<Id, DropResult<Value>>,
}

impl<Id: Eq + Hash, Value: ValueView + Clone> PreloaderResult<Id, Value> {
  pub fn new(values_by_id: HashMap<Id, DropResult<Value>>) -> Self {
    Self { values_by_id }
  }

  pub fn get(&self, id: &Id) -> DropResult<Value> {
    self.values_by_id.get(id).cloned().unwrap_or_default()
  }

  #[allow(dead_code)]
  pub fn expect_value(&self, id: &Id) -> Result<Value, DropError> {
    self
      .get(id)
      .get_inner()
      .cloned()
      .ok_or_else(|| DropError::ExpectedEntityNotFound(std::any::type_name::<Value>().to_string()))
  }

  pub fn all_values<'a>(&'a self) -> Box<dyn Iterator<Item = &'a DropResult<Value>> + 'a> {
    Box::new(self.values_by_id.values())
  }

  pub fn iter_mut(&mut self) -> IterMut<Id, DropResult<Value>> {
    self.values_by_id.iter_mut()
  }

  pub fn all_values_unwrapped<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Value> + 'a> {
    Box::new(
      self
        .all_values()
        .into_iter()
        .map(|value| value.get_inner().unwrap()),
    )
  }

  pub fn extend(&mut self, items: &mut dyn Iterator<Item = (Id, DropResult<Value>)>) {
    self.values_by_id.extend(items)
  }
}

impl<Id: Eq + Hash, InnerValue: ValueView + Clone> PreloaderResult<Id, Vec<InnerValue>> {
  pub fn all_values_flat<'a>(&'a self) -> Box<dyn Iterator<Item = &'a InnerValue> + 'a> {
    Box::new(self.all_values().flat_map(|v| v.get_inner().unwrap()))
  }
}

impl<Id: Eq + Hash, InnerValue: ValueView + Clone>
  PreloaderResult<Id, Vec<ArcValueView<InnerValue>>>
{
  pub fn all_values_flat_unwrapped<'a>(&'a self) -> Box<dyn Iterator<Item = &'a InnerValue> + 'a> {
    Box::new(self.all_values_flat().map(|v| v.as_ref()))
  }
}
