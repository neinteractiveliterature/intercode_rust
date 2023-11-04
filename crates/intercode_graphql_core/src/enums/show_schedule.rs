use async_graphql::Enum;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum ShowSchedule {
  #[graphql(name = "no")]
  No,
  #[graphql(name = "priv")]
  Priv,
  #[graphql(name = "gms")]
  GMs,
  #[graphql(name = "yes")]
  Yes,
}

impl TryFrom<&str> for ShowSchedule {
  type Error = async_graphql::Error;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "no" => Ok(ShowSchedule::No),
      "priv" => Ok(ShowSchedule::Priv),
      "gms" => Ok(ShowSchedule::GMs),
      "yes" => Ok(ShowSchedule::Yes),
      _ => Err(Self::Error::new(format!(
        "Unknown show schedule value: {}",
        value
      ))),
    }
  }
}
