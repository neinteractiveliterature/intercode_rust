use liquid::ValueView;

use crate::DropResult;

#[derive(Debug, Clone)]
pub struct ExtendedDropResult<T: liquid::model::ValueView> {
  pub drop_result: DropResult<T>,
  pub extensions: liquid::model::Object,
}

impl<T: liquid::model::ValueView> liquid::ValueView for ExtendedDropResult<T> {
  fn as_debug(&self) -> &dyn std::fmt::Debug {
    todo!()
  }

  fn as_object(&self) -> Option<&dyn liquid::ObjectView> {
    Some(self)
  }

  fn render(&self) -> liquid::model::DisplayCow<'_> {
    self.drop_result.render()
  }

  fn source(&self) -> liquid::model::DisplayCow<'_> {
    self.drop_result.source()
  }

  fn type_name(&self) -> &'static str {
    self.drop_result.type_name()
  }

  fn query_state(&self, state: liquid::model::State) -> bool {
    self.drop_result.query_state(state)
  }

  fn to_kstr(&self) -> liquid::model::KStringCow<'_> {
    self.drop_result.to_kstr()
  }

  fn to_value(&self) -> liquid_core::Value {
    todo!()
  }
}

impl<T: liquid::model::ValueView> liquid::ObjectView for ExtendedDropResult<T> {
  fn as_value(&self) -> &dyn ValueView {
    self
  }

  fn size(&self) -> i64 {
    self.drop_result.as_object().unwrap().size() + self.extensions.size()
  }

  fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = liquid::model::KStringCow<'k>> + 'k> {
    let drop_result_keys = self.drop_result.as_object().unwrap().keys();
    let extension_keys: Box<dyn Iterator<Item = liquid::model::KStringCow<'k>> + 'k> =
      Box::new(self.extensions.keys().map(|key| key.into()));

    Box::new(drop_result_keys.chain(extension_keys))
  }

  fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn ValueView> + 'k> {
    let drop_result_values = self.drop_result.as_object().unwrap().values();
    let extension_values: Box<dyn Iterator<Item = &'k dyn ValueView> + 'k> =
      Box::new(self.extensions.values().map(|value| value.as_view()));

    Box::new(drop_result_values.chain(extension_values))
  }

  fn iter<'k>(
    &'k self,
  ) -> Box<dyn Iterator<Item = (liquid::model::KStringCow<'k>, &'k dyn ValueView)> + 'k> {
    let drop_result_iter = self.drop_result.as_object().unwrap().iter();
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
    self.drop_result.as_object().unwrap().contains_key(index) || self.extensions.contains_key(index)
  }

  fn get<'s>(&'s self, index: &str) -> Option<&'s dyn ValueView> {
    self
      .drop_result
      .as_object()
      .unwrap()
      .get(index)
      .or_else(|| self.extensions.get(index).map(|value| value.as_view()))
  }
}

impl<T: liquid::model::ValueView> Extend<(liquid::model::KString, liquid::model::Value)>
  for ExtendedDropResult<T>
{
  fn extend<I: IntoIterator<Item = (liquid::model::KString, liquid::model::Value)>>(
    &mut self,
    iter: I,
  ) {
    self.extensions.extend(iter)
  }
}
