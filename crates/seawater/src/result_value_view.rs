use std::fmt::Debug;

use liquid::ValueView;

use crate::DropResultTrait;

#[derive(Debug, Clone)]
pub struct ResultValueView<V: ValueView, E: Debug> {
  result: Result<V, E>,
  error_message: String,
}

impl<V: ValueView, E: Debug> ResultValueView<V, E> {
  pub fn new(result: Result<V, E>) -> Self {
    let error_message = result
      .as_ref()
      .err()
      .map(|err| format!("{:?}", err))
      .unwrap_or_default();

    ResultValueView {
      result,
      error_message,
    }
  }

  fn as_value_view(&self) -> &dyn ValueView {
    match &self.result {
      Ok(value) => value,
      Err(_) => &self.error_message,
    }
  }

  pub fn as_result(&self) -> &Result<V, E> {
    &self.result
  }
}

impl<V: ValueView, E: Debug> ValueView for ResultValueView<V, E> {
  fn as_debug(&self) -> &dyn std::fmt::Debug {
    self
  }

  fn render(&self) -> liquid::model::DisplayCow<'_> {
    self.as_value_view().render()
  }

  fn source(&self) -> liquid::model::DisplayCow<'_> {
    self.as_value_view().source()
  }

  fn type_name(&self) -> &'static str {
    self.as_value_view().type_name()
  }

  fn query_state(&self, state: liquid::model::State) -> bool {
    self.as_value_view().query_state(state)
  }

  fn to_kstr(&self) -> liquid::model::KStringCow<'_> {
    self.as_value_view().to_kstr()
  }

  fn to_value(&self) -> liquid_core::Value {
    self.as_value_view().to_value()
  }
}

impl<V: ValueView, E: Debug> From<Result<V, E>> for ResultValueView<V, E> {
  fn from(result: Result<V, E>) -> Self {
    ResultValueView::new(result)
  }
}

impl<V: ValueView + Clone + Send + Sync, E: Debug + Send + Sync + Clone>
  DropResultTrait<ResultValueView<V, E>> for ResultValueView<V, E>
{
  fn get_inner<'a>(&'a self) -> Box<dyn std::ops::Deref<Target = ResultValueView<V, E>> + 'a> {
    Box::new(self)
  }
}
