//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "runs")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub event_id: Option<i64>,
  pub starts_at: Option<DateTime>,
  pub title_suffix: Option<String>,
  #[sea_orm(column_type = "Text", nullable)]
  pub schedule_note: Option<String>,
  pub updated_by_id: Option<i64>,
  pub created_at: Option<DateTime>,
  pub updated_at: Option<DateTime>,
  #[sea_orm(column_type = "Custom(\"tsrange\".to_owned())")]
  pub timespan_tsrange: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::users::Entity",
    from = "Column::UpdatedById",
    to = "super::users::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Users,
  #[sea_orm(
    belongs_to = "super::events::Entity",
    from = "Column::EventId",
    to = "super::events::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Events,
  #[sea_orm(has_many = "super::signup_changes::Entity")]
  SignupChanges,
  #[sea_orm(has_many = "super::signups::Entity")]
  Signups,
  #[sea_orm(has_many = "super::signup_requests::Entity")]
  SignupRequests,
}

impl Related<super::users::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Users.def()
  }
}

impl Related<super::events::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Events.def()
  }
}

impl Related<super::signup_changes::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::SignupChanges.def()
  }
}

impl Related<super::signups::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Signups.def()
  }
}

impl Related<super::signup_requests::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::SignupRequests.def()
  }
}

impl Related<super::rooms::Entity> for Entity {
  fn to() -> RelationDef {
    super::rooms_runs::Relation::Rooms.def()
  }
  fn via() -> Option<RelationDef> {
    Some(super::rooms_runs::Relation::Runs.def().rev())
  }
}

impl ActiveModelBehavior for ActiveModel {}
