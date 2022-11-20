//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "ticket_types")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub convention_id: Option<i64>,
  #[sea_orm(column_type = "Text")]
  pub name: String,
  #[sea_orm(column_type = "Text", nullable)]
  pub description: Option<String>,
  pub created_at: DateTime,
  pub updated_at: DateTime,
  pub counts_towards_convention_maximum: bool,
  pub maximum_event_provided_tickets: i32,
  pub allows_event_signups: bool,
  pub event_id: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::events::Entity",
    from = "Column::EventId",
    to = "super::events::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Events,
  #[sea_orm(
    belongs_to = "super::conventions::Entity",
    from = "Column::ConventionId",
    to = "super::conventions::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Conventions,
  #[sea_orm(has_many = "super::products::Entity")]
  Products,
  #[sea_orm(has_many = "super::tickets::Entity")]
  Tickets,
  #[sea_orm(has_many = "super::maximum_event_provided_tickets_overrides::Entity")]
  MaximumEventProvidedTicketsOverrides,
}

impl Related<super::events::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Events.def()
  }
}

impl Related<super::conventions::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Conventions.def()
  }
}

impl Related<super::products::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Products.def()
  }
}

impl Related<super::tickets::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Tickets.def()
  }
}

impl Related<super::maximum_event_provided_tickets_overrides::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::MaximumEventProvidedTicketsOverrides.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
