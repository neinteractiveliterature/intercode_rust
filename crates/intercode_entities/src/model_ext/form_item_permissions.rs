use sea_orm::{DeriveActiveEnum, EnumIter};
use serde::{Deserialize, Serialize};

#[derive(
  Clone,
  Copy,
  Debug,
  Default,
  PartialEq,
  Eq,
  PartialOrd,
  Ord,
  Serialize,
  Deserialize,
  EnumIter,
  DeriveActiveEnum,
  async_graphql::Enum,
)]
#[serde(rename_all = "snake_case")]
#[sea_orm(rs_type = "String", db_type = "String(None)")]
pub enum FormItemRole {
  #[sea_orm(string_value = "normal")]
  #[graphql(name = "normal")]
  #[default]
  Normal,
  #[sea_orm(string_value = "confirmed_attendee")]
  #[graphql(name = "confirmed_attendee")]
  ConfirmedAttendee,
  #[sea_orm(string_value = "team_member")]
  #[graphql(name = "team_member")]
  TeamMember,
  #[sea_orm(string_value = "all_profiles_basic_access")]
  #[graphql(name = "all_profiles_basic_access")]
  AllProfilesBasicAccess,
  #[sea_orm(string_value = "admin")]
  #[graphql(name = "admin")]
  Admin,
}
