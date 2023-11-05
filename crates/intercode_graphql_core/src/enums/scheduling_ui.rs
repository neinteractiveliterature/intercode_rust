use async_graphql::Enum;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum SchedulingUi {
  #[graphql(name = "regular")]
  Regular,
  #[graphql(name = "recurring")]
  Recurring,
  #[graphql(name = "single_run")]
  SingleRun,
}

impl TryFrom<&str> for SchedulingUi {
  type Error = async_graphql::Error;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "regular" => Ok(Self::Regular),
      "recurring" => Ok(Self::Recurring),
      "single_run" => Ok(Self::SingleRun),
      _ => Err(Self::Error::new(format!(
        "Unknown scheduling UI: {}",
        value
      ))),
    }
  }
}
