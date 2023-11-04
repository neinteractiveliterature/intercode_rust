use async_graphql::Enum;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum SchedulingUI {
  #[graphql(name = "regular")]
  Regular,
  #[graphql(name = "recurring")]
  Recurring,
  #[graphql(name = "single_run")]
  SingleRun,
}

impl TryFrom<&str> for SchedulingUI {
  type Error = async_graphql::Error;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "regular" => Ok(SchedulingUI::Regular),
      "recurring" => Ok(SchedulingUI::Recurring),
      "single_run" => Ok(SchedulingUI::SingleRun),
      _ => Err(Self::Error::new(format!(
        "Unknown scheduling UI: {}",
        value
      ))),
    }
  }
}
