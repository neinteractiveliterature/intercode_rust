use crate::{DropError, DropResult};
use liquid::ValueView;
use std::{
  collections::{hash_map::IterMut, HashMap},
  hash::Hash,
};

#[derive(Debug)]
pub struct PreloaderResult<Id: Eq + Hash, Value: ValueView + Clone + Send + Sync + 'static> {
  values_by_id: HashMap<Id, DropResult<Value>>,
}

impl<Id: Eq + Hash, Value: ValueView + Clone + Send + Sync + 'static> PreloaderResult<Id, Value>
where
  DropResult<Value>: Clone,
{
  pub fn new(values_by_id: HashMap<Id, DropResult<Value>>) -> Self {
    Self { values_by_id }
  }

  pub fn get(&self, id: Id) -> DropResult<Value> {
    self
      .values_by_id
      .get(&id)
      .and_then(|drop_result| drop_result.get_inner_cloned())
      .into()
  }

  #[allow(dead_code)]
  pub fn expect_value(&self, id: Id) -> Result<Value, DropError> {
    self
      .values_by_id
      .get(&id)
      .and_then(|drop_result| drop_result.get_inner_cloned())
      .ok_or_else(|| DropError::ExpectedEntityNotFound(std::any::type_name::<Value>().to_string()))
  }

  pub fn all_values(&self) -> Box<dyn Iterator<Item = &DropResult<Value>> + '_> {
    Box::new(self.values_by_id.values())
  }

  pub fn iter_mut(&mut self) -> IterMut<Id, DropResult<Value>> {
    self.values_by_id.iter_mut()
  }

  pub fn all_values_unwrapped<'a>(&'a self) -> Box<dyn Iterator<Item = Value> + 'a> {
    Box::new(
      self
        .all_values()
        .map(|value| value.get_inner_cloned().unwrap()),
    )
  }

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
