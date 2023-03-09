use std::borrow::Cow;

use liquid::ValueView;

#[derive(Debug, Clone)]
pub enum OptionalValueView<V: ValueView> {
  Some(V),
  None,
}

impl<V: ValueView> OptionalValueView<V> {
  fn as_value_view(&self) -> &dyn ValueView {
    match self {
      OptionalValueView::Some(value) => value,
      OptionalValueView::None => &liquid::model::Value::Nil,
    }
  }

  pub fn as_option(&self) -> Option<&V> {
    match self {
      OptionalValueView::Some(value) => Some(value),
      OptionalValueView::None => None,
    }
  }

  pub fn cloned(&self) -> OptionalValueView<V>
  where
    V: Clone,
  {
    match self {
      OptionalValueView::Some(value) => OptionalValueView::Some(value.clone()),
      OptionalValueView::None => OptionalValueView::None,
    }
  }
}

impl<V: ValueView> ValueView for OptionalValueView<V> {
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

impl<V: ValueView> From<Option<V>> for OptionalValueView<V> {
  fn from(value: Option<V>) -> Self {
    match value {
      Some(value) => OptionalValueView::Some(value),
      None => OptionalValueView::None,
    }
  }
}

impl<V: ValueView> From<OptionalValueView<V>> for Option<V> {
  fn from(value: OptionalValueView<V>) -> Self {
    match value {
      OptionalValueView::Some(value) => Some(value),
      OptionalValueView::None => None,
    }
  }
}

impl<V: ValueView + Clone> From<Option<Cow<'_, V>>> for OptionalValueView<V> {
  fn from(value: Option<Cow<V>>) -> Self {
    match value {
      Some(value) => OptionalValueView::Some(value.into_owned()),
      None => OptionalValueView::None,
    }
  }
}
