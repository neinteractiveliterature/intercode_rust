use std::fmt::Debug;

use liquid::ValueView;

use crate::DropResult;

#[derive(Clone)]
pub struct ExtendedDropResult<'a, T: ValueView + Clone> {
  pub drop_result: &'a DropResult<T>,
  pub extensions: liquid::model::Object,
}

impl<'a, T: ValueView + Clone> ExtendedDropResult<'a, T> {
  pub fn new(drop_result: &'a DropResult<T>, extensions: liquid::model::Object) -> Self {
    ExtendedDropResult {
      drop_result,
      extensions,
    }
  }
}

impl<'a, T: ValueView + Clone> Debug for ExtendedDropResult<'a, T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ExtendedDropResult")
      .field("drop_result", &self.drop_result.get_inner())
      .field("extensions", &self.extensions)
      .finish()
  }
}

impl<'a, T: ValueView + Clone> ValueView for ExtendedDropResult<'a, T> {
  fn as_debug(&self) -> &dyn std::fmt::Debug {
    todo!()
  }

  fn as_object(&self) -> Option<&dyn liquid::ObjectView> {
    Some(self)
  }

  fn render(&self) -> liquid::model::DisplayCow<'_> {
    self.drop_result.get_value().render()
  }

  fn source(&self) -> liquid::model::DisplayCow<'_> {
    self.drop_result.get_value().source()
  }

  fn type_name(&self) -> &'static str {
    self.drop_result.get_value().type_name()
  }

  fn query_state(&self, state: liquid::model::State) -> bool {
    self.drop_result.get_value().query_state(state)
  }

  fn to_kstr(&self) -> liquid::model::KStringCow<'_> {
    self.drop_result.get_value().to_kstr()
  }

  fn to_value(&self) -> liquid_core::Value {
    todo!()
  }
}

impl<'a, T: ValueView + Clone> liquid::ObjectView for ExtendedDropResult<'a, T> {
  fn as_value(&self) -> &dyn ValueView {
    self
  }

  fn size(&self) -> i64 {
    self.drop_result.get_inner().as_object().unwrap().size() + self.extensions.size()
  }

  fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = liquid::model::KStringCow<'k>> + 'k> {
    let drop_result_keys = self.drop_result.get_value().as_object().unwrap().keys();
    let extension_keys: Box<dyn Iterator<Item = liquid::model::KStringCow<'k>> + 'k> =
      Box::new(self.extensions.keys().map(|key| key.into()));

    Box::new(drop_result_keys.chain(extension_keys))
  }

  fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn ValueView> + 'k> {
    let drop_result_values = self.drop_result.get_value().as_object().unwrap().values();
    let extension_values: Box<dyn Iterator<Item = &'k dyn ValueView> + 'k> =
      Box::new(self.extensions.values().map(|value| value.as_view()));

    Box::new(drop_result_values.chain(extension_values))
  }

  fn iter<'k>(
    &'k self,
  ) -> Box<dyn Iterator<Item = (liquid::model::KStringCow<'k>, &'k dyn ValueView)> + 'k> {
    let drop_result_iter = self.drop_result.get_value().as_object().unwrap().iter();
    let extension_iter: Box<
      dyn Iterator<Item = (liquid::model::KStringCow<'k>, &'k dyn ValueView)> + 'k,
    > = Box::new(
      self
        .extensions
        .iter()
        .map(|(key, value)| (key.into(), value.as_view())),
    );

    Box::new(drop_result_iter.chain(extension_iter))
  }

  fn contains_key(&self, index: &str) -> bool {
    self
      .drop_result
      .get_inner()
      .as_object()
      .unwrap()
      .contains_key(index)
      || self.extensions.contains_key(index)
  }

  fn get<'s>(&'s self, index: &str) -> Option<&'s dyn ValueView> {
    self
      .drop_result
      .get_value()
      .as_object()
      .unwrap()
      .get(index)
      .or_else(|| self.extensions.get(index).map(|value| value.as_view()))
  }
}

impl<'a, T: ValueView + Clone> Extend<(liquid::model::KString, liquid::model::Value)>
  for ExtendedDropResult<'a, T>
{
  fn extend<I: IntoIterator<Item = (liquid::model::KString, liquid::model::Value)>>(
    &mut self,
    iter: I,
  ) {
    self.extensions.extend(liquid::Object::from_iter(iter));
  }
}
