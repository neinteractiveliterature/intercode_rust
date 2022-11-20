//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "maximum_event_provided_tickets_overrides")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub event_id: Option<i64>,
  pub ticket_type_id: Option<i64>,
  pub override_value: Option<i32>,
  pub created_at: DateTime,
  pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::ticket_types::Entity",
    from = "Column::TicketTypeId",
    to = "super::ticket_types::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  TicketTypes,
  #[sea_orm(
    belongs_to = "super::events::Entity",
    from = "Column::EventId",
    to = "super::events::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Events,
}

impl Related<super::ticket_types::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::TicketTypes.def()
  }
}

impl Related<super::events::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Events.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
