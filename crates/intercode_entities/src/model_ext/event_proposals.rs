use sea_orm::{DeriveActiveEnum, EnumIter};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, EnumIter, DeriveActiveEnum, Clone, PartialEq, Eq, Debug, Hash)]
#[serde(rename_all = "snake_case")]
#[sea_orm(rs_type = "String", db_type = "String(Some(1))")]
pub enum EventProposalStatus {
  #[sea_orm(string_value = "draft")]
  Draft,
  #[sea_orm(string_value = "proposed")]
  Proposed,
  #[sea_orm(string_value = "reviewing")]
  Reviewing,
  #[sea_orm(string_value = "tentative_accept")]
  TentativeAccept,
  #[sea_orm(string_value = "accepted")]
  Accepted,
  #[sea_orm(string_value = "rejected")]
  Rejected,
  #[sea_orm(string_value = "withdrawn")]
  Withdrawn,
}
