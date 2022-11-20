//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "event_categories")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub convention_id: i64,
  #[sea_orm(column_type = "Text")]
  pub name: String,
  #[sea_orm(column_type = "Text")]
  pub team_member_name: String,
  #[sea_orm(column_type = "Text")]
  pub scheduling_ui: String,
  #[sea_orm(column_type = "Text")]
  pub default_color: String,
  #[sea_orm(column_type = "Text")]
  pub full_color: String,
  #[sea_orm(column_type = "Text")]
  pub signed_up_color: String,
  pub can_provide_tickets: bool,
  pub event_form_id: i64,
  pub event_proposal_form_id: Option<i64>,
  pub created_at: DateTime,
  pub updated_at: DateTime,
  pub department_id: Option<i64>,
  #[sea_orm(column_type = "Text", nullable)]
  pub proposal_description: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::conventions::Entity",
    from = "Column::ConventionId",
    to = "super::conventions::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Conventions,
  #[sea_orm(
    belongs_to = "super::forms::Entity",
    from = "Column::EventFormId",
    to = "super::forms::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Forms2,
  #[sea_orm(
    belongs_to = "super::forms::Entity",
    from = "Column::EventProposalFormId",
    to = "super::forms::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Forms1,
  #[sea_orm(
    belongs_to = "super::departments::Entity",
    from = "Column::DepartmentId",
    to = "super::departments::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Departments,
  #[sea_orm(has_many = "super::event_proposals::Entity")]
  EventProposals,
  #[sea_orm(has_many = "super::events::Entity")]
  Events,
  #[sea_orm(has_many = "super::permissions::Entity")]
  Permissions,
}

impl Related<super::conventions::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Conventions.def()
  }
}

impl Related<super::departments::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Departments.def()
  }
}

impl Related<super::event_proposals::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::EventProposals.def()
  }
}

impl Related<super::events::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Events.def()
  }
}

impl Related<super::permissions::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Permissions.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
