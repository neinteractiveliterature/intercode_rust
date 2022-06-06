//! SeaORM Entity. Generated by sea-orm-codegen 0.7.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "conventions")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub show_schedule: String,
  pub accepting_proposals: Option<bool>,
  pub updated_by_id: Option<i64>,
  pub created_at: Option<DateTime>,
  pub updated_at: Option<DateTime>,
  pub starts_at: Option<DateTime>,
  pub ends_at: Option<DateTime>,
  pub root_page_id: Option<i64>,
  pub name: Option<String>,
  pub domain: String,
  pub timezone_name: Option<String>,
  pub maximum_event_signups: Option<Json>,
  pub maximum_tickets: Option<i32>,
  pub default_layout_id: Option<i64>,
  pub user_con_profile_form_id: Option<i64>,
  pub ticket_name: String,
  #[sea_orm(column_type = "Text", nullable)]
  pub event_mailing_list_domain: Option<String>,
  #[sea_orm(column_type = "Text", nullable)]
  pub clickwrap_agreement: Option<String>,
  pub show_event_list: String,
  pub organization_id: Option<i64>,
  pub ticket_mode: String,
  pub site_mode: String,
  pub signup_mode: String,
  pub signup_requests_open: bool,
  #[sea_orm(column_type = "Text")]
  pub email_from: String,
  pub catch_all_staff_position_id: Option<i64>,
  pub email_mode: String,
  pub canceled: bool,
  pub location: Option<Json>,
  pub timezone_mode: String,
  pub hidden: bool,
  pub language: String,
  #[sea_orm(column_type = "Text", nullable)]
  pub stripe_account_id: Option<String>,
  pub stripe_account_ready_to_charge: bool,
  #[sea_orm(column_type = "Text", nullable)]
  pub open_graph_image: Option<String>,
  #[sea_orm(column_type = "Text", nullable)]
  pub favicon: Option<String>,
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
    belongs_to = "super::pages::Entity",
    from = "Column::RootPageId",
    to = "super::pages::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Pages,
  #[sea_orm(
    belongs_to = "super::forms::Entity",
    from = "Column::UserConProfileFormId",
    to = "super::forms::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Forms,
  #[sea_orm(
    belongs_to = "super::organizations::Entity",
    from = "Column::OrganizationId",
    to = "super::organizations::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Organizations,
  #[sea_orm(
    belongs_to = "super::staff_positions::Entity",
    from = "Column::CatchAllStaffPositionId",
    to = "super::staff_positions::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  StaffPositions,
  #[sea_orm(
    belongs_to = "super::cms_layouts::Entity",
    from = "Column::DefaultLayoutId",
    to = "super::cms_layouts::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  CmsLayouts,
  #[sea_orm(has_many = "super::coupons::Entity")]
  Coupons,
  #[sea_orm(has_many = "super::departments::Entity")]
  Departments,
  #[sea_orm(has_many = "super::event_categories::Entity")]
  EventCategories,
  #[sea_orm(has_many = "super::events::Entity")]
  Events,
  #[sea_orm(has_many = "super::event_proposals::Entity")]
  EventProposals,
  #[sea_orm(has_many = "super::notification_templates::Entity")]
  NotificationTemplates,
  #[sea_orm(has_many = "super::rooms::Entity")]
  Rooms,
  #[sea_orm(has_many = "super::user_activity_alerts::Entity")]
  UserActivityAlerts,
  #[sea_orm(has_many = "super::products::Entity")]
  Products,
  #[sea_orm(has_many = "super::permissions::Entity")]
  Permissions,
  #[sea_orm(has_many = "super::ticket_types::Entity")]
  TicketTypes,
  #[sea_orm(has_many = "super::user_con_profiles::Entity")]
  UserConProfiles,
}

impl Related<super::users::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Users.def()
  }
}

impl Related<super::pages::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Pages.def()
  }
}

impl Related<super::forms::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Forms.def()
  }
}

impl Related<super::organizations::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Organizations.def()
  }
}

impl Related<super::staff_positions::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::StaffPositions.def()
  }
}

impl Related<super::cms_layouts::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::CmsLayouts.def()
  }
}

impl Related<super::coupons::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Coupons.def()
  }
}

impl Related<super::departments::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Departments.def()
  }
}

impl Related<super::event_categories::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::EventCategories.def()
  }
}

impl Related<super::events::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Events.def()
  }
}

impl Related<super::event_proposals::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::EventProposals.def()
  }
}

impl Related<super::notification_templates::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::NotificationTemplates.def()
  }
}

impl Related<super::rooms::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Rooms.def()
  }
}

impl Related<super::user_activity_alerts::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::UserActivityAlerts.def()
  }
}

impl Related<super::products::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Products.def()
  }
}

impl Related<super::permissions::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Permissions.def()
  }
}

impl Related<super::ticket_types::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::TicketTypes.def()
  }
}

impl Related<super::user_con_profiles::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::UserConProfiles.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
