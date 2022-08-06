use std::sync::Arc;

use liquid::ObjectView;

use crate::ExtendedDropResult;

#[derive(Debug, Clone)]
pub struct DropResult<'a> {
  pub value: Arc<dyn liquid::ValueView + 'a>,
}

impl<'a> DropResult<'a> {
  pub fn extend(&self, extensions: liquid::model::Object) -> ExtendedDropResult<'a> {
    ExtendedDropResult {
      drop_result: self.clone(),
      extensions,
    }
  }
}

impl Default for DropResult<'_> {
  fn default() -> Self {
    DropResult::new(liquid::model::Value::Nil)
  }
}

impl<'a> liquid::ValueView for DropResult<'a> {
  fn as_debug(&self) -> &dyn std::fmt::Debug {
    self.value.as_debug()
  }

  fn as_object(&self) -> Option<&dyn liquid::ObjectView> {
    Some(self)
  }

  fn render(&self) -> liquid::model::DisplayCow<'_> {
    self.value.render()
  }

  fn source(&self) -> liquid::model::DisplayCow<'_> {
    self.value.source()
  }

  fn type_name(&self) -> &'static str {
    self.value.type_name()
  }

  fn query_state(&self, state: liquid::model::State) -> bool {
    self.value.query_state(state)
  }

  fn to_kstr(&self) -> liquid::model::KStringCow<'_> {
    self.value.to_kstr()
  }

  fn to_value(&self) -> liquid_core::Value {
    self.value.to_value()
  }
}

impl<'a> ObjectView for DropResult<'a> {
  fn as_value(&self) -> &dyn liquid::ValueView {
    self
  }

  fn size(&self) -> i64 {
    self.value.as_object().unwrap().size()
  }

  fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = liquid::model::KStringCow<'k>> + 'k> {
    self.value.as_object().unwrap().keys()
  }

  fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn liquid::ValueView> + 'k> {
    self.value.as_object().unwrap().values()
  }

  fn iter<'k>(
    &'k self,
  ) -> Box<dyn Iterator<Item = (liquid::model::KStringCow<'k>, &'k dyn liquid::ValueView)> + 'k> {
    self.value.as_object().unwrap().iter()
  }

  fn contains_key(&self, index: &str) -> bool {
    self.value.as_object().unwrap().contains_key(index)
  }

  fn get<'s>(&'s self, index: &str) -> Option<&'s dyn liquid::ValueView> {
    self.value.as_object().unwrap().get(index)
  }
}

impl<'a> DropResult<'a> {
  pub fn new<T: liquid::ValueView + 'a>(value_view: T) -> Self {
    DropResult {
      value: Arc::new(value_view),
    }
  }
}

macro_rules! drop_result_value_converter {
  ($t: ty) => {
    impl<'a> From<$t> for DropResult<'a> {
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

impl<'a> From<&str> for DropResult<'a> {
  fn from(string: &str) -> Self {
    DropResult::new(string.to_owned())
  }
}

impl<'a> From<&'a dyn liquid::ValueView> for DropResult<'a> {
  fn from(value_view_ref: &'a dyn liquid::ValueView) -> Self {
    DropResult::new(value_view_ref)
  }
}

impl<'a> From<&serde_json::Value> for DropResult<'a> {
  fn from(value: &serde_json::Value) -> Self {
    DropResult::new(liquid::model::to_value(value).unwrap())
  }
}

impl<'a, T> From<Option<T>> for DropResult<'a>
where
  T: Into<DropResult<'a>>,
{
  fn from(option: Option<T>) -> Self {
    option.map(|value| value.into()).unwrap_or_default()
  }
}

impl<'a, T, E> From<Result<T, E>> for DropResult<'a>
where
  T: Into<DropResult<'a>>,
{
  fn from(result: Result<T, E>) -> Self {
    result.map(|value| value.into()).unwrap_or_default()
  }
}

impl<'a, T> From<Vec<T>> for DropResult<'a>
where
  T: Into<DropResult<'a>>,
{
  fn from(values: Vec<T>) -> Self {
    let converted_values: Vec<DropResult> = values
      .into_iter()
      .map(|value| value.into())
      .collect::<Vec<DropResult>>();

    DropResult::new(converted_values)
  }
}
