#[macro_use]
mod entities_by_id_loader;
#[macro_use]
mod entities_by_relation_loader;
#[macro_use]
mod entities_by_link_loader;
pub mod expect;

use std::env;
use std::sync::Arc;
use std::time::Duration;

use async_graphql::dataloader::DataLoader;
pub use entities_by_id_loader::*;
pub use entities_by_link_loader::*;
pub use entities_by_relation_loader::*;
use sea_orm::RelationTrait;
use sea_orm::{Linked, RelationDef};

use intercode_entities::{cms_navigation_items, conventions, pages};
use intercode_entities::{event_categories, users};
use intercode_entities::{events, ticket_types};
use intercode_entities::{order_entries, orders, team_members};
use intercode_entities::{products, staff_positions};
use intercode_entities::{signups, user_con_profiles};
use intercode_entities::{staff_positions_user_con_profiles, tickets};

#[derive(Debug, Clone)]
pub struct UserConProfileToStaffPositions;

impl Linked for UserConProfileToStaffPositions {
  type FromEntity = user_con_profiles::Entity;
  type ToEntity = staff_positions::Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![
      staff_positions_user_con_profiles::Relation::UserConProfiles
        .def()
        .rev(),
      staff_positions_user_con_profiles::Relation::StaffPositions.def(),
    ]
  }
}

#[derive(Debug, Clone)]
pub struct CmsNavigationItemToCmsNavigationSection;

impl Linked for CmsNavigationItemToCmsNavigationSection {
  type FromEntity = cms_navigation_items::Entity;
  type ToEntity = cms_navigation_items::Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![cms_navigation_items::Relation::SelfRef.def()]
  }
}

impl_to_entity_link_loader!(
  cms_navigation_items::Entity,
  CmsNavigationItemToCmsNavigationSection,
  cms_navigation_items::Entity,
  cms_navigation_items::PrimaryKey::Id
);

impl_to_entity_relation_loader!(
  conventions::Entity,
  event_categories::Entity,
  conventions::PrimaryKey::Id
);

impl_to_entity_relation_loader!(
  cms_navigation_items::Entity,
  pages::Entity,
  cms_navigation_items::PrimaryKey::Id
);

impl_to_entity_relation_loader!(
  conventions::Entity,
  ticket_types::Entity,
  conventions::PrimaryKey::Id
);

impl_to_entity_id_loader!(conventions::Entity, conventions::PrimaryKey::Id);
impl_to_entity_id_loader!(staff_positions::Entity, staff_positions::PrimaryKey::Id);
impl_to_entity_id_loader!(team_members::Entity, team_members::PrimaryKey::Id);
impl_to_entity_id_loader!(users::Entity, users::PrimaryKey::Id);

impl_to_entity_relation_loader!(
  orders::Entity,
  order_entries::Entity,
  orders::PrimaryKey::Id
);

impl_to_entity_relation_loader!(
  team_members::Entity,
  events::Entity,
  team_members::PrimaryKey::Id
);

impl_to_entity_relation_loader!(
  ticket_types::Entity,
  products::Entity,
  ticket_types::PrimaryKey::Id
);

impl_to_entity_relation_loader!(
  user_con_profiles::Entity,
  team_members::Entity,
  user_con_profiles::PrimaryKey::Id
);

impl_to_entity_relation_loader!(
  user_con_profiles::Entity,
  signups::Entity,
  user_con_profiles::PrimaryKey::Id
);

impl_to_entity_link_loader!(
  user_con_profiles::Entity,
  UserConProfileToStaffPositions,
  staff_positions::Entity,
  user_con_profiles::PrimaryKey::Id
);

impl_to_entity_relation_loader!(
  user_con_profiles::Entity,
  tickets::Entity,
  user_con_profiles::PrimaryKey::Id
);

impl_to_entity_relation_loader!(
  user_con_profiles::Entity,
  users::Entity,
  user_con_profiles::PrimaryKey::Id
);

pub struct LoaderManager {
  db: Arc<sea_orm::DatabaseConnection>,
  pub cms_navigation_item_page: DataLoader<
    EntityRelationLoader<
      cms_navigation_items::Entity,
      pages::Entity,
      cms_navigation_items::PrimaryKey,
    >,
  >,
  pub cms_navigation_item_section: DataLoader<
    EntityLinkLoader<
      cms_navigation_items::Entity,
      CmsNavigationItemToCmsNavigationSection,
      cms_navigation_items::Entity,
      cms_navigation_items::PrimaryKey,
    >,
  >,
  pub convention_event_categories: DataLoader<
    EntityRelationLoader<conventions::Entity, event_categories::Entity, conventions::PrimaryKey>,
  >,
  pub convention_ticket_types: DataLoader<
    EntityRelationLoader<conventions::Entity, ticket_types::Entity, conventions::PrimaryKey>,
  >,
  pub conventions_by_id: DataLoader<EntityIdLoader<conventions::Entity, conventions::PrimaryKey>>,
  pub order_order_entries:
    DataLoader<EntityRelationLoader<orders::Entity, order_entries::Entity, orders::PrimaryKey>>,
  pub staff_positions_by_id:
    DataLoader<EntityIdLoader<staff_positions::Entity, staff_positions::PrimaryKey>>,
  pub team_member_event: DataLoader<
    EntityRelationLoader<team_members::Entity, events::Entity, team_members::PrimaryKey>,
  >,
  pub team_members_by_id:
    DataLoader<EntityIdLoader<team_members::Entity, team_members::PrimaryKey>>,
  pub ticket_type_providing_products: DataLoader<
    EntityRelationLoader<ticket_types::Entity, products::Entity, ticket_types::PrimaryKey>,
  >,
  pub user_con_profile_signups: DataLoader<
    EntityRelationLoader<user_con_profiles::Entity, signups::Entity, user_con_profiles::PrimaryKey>,
  >,
  pub user_con_profile_staff_positions: DataLoader<
    EntityLinkLoader<
      user_con_profiles::Entity,
      UserConProfileToStaffPositions,
      staff_positions::Entity,
      user_con_profiles::PrimaryKey,
    >,
  >,
  pub user_con_profile_ticket: DataLoader<
    EntityRelationLoader<user_con_profiles::Entity, tickets::Entity, user_con_profiles::PrimaryKey>,
  >,
  pub user_con_profile_user: DataLoader<
    EntityRelationLoader<user_con_profiles::Entity, users::Entity, user_con_profiles::PrimaryKey>,
  >,
  pub user_con_profile_team_members: DataLoader<
    EntityRelationLoader<
      user_con_profiles::Entity,
      team_members::Entity,
      user_con_profiles::PrimaryKey,
    >,
  >,
  pub users_by_id: DataLoader<EntityIdLoader<users::Entity, users::PrimaryKey>>,
}

impl LoaderManager {
  pub fn new(db: &Arc<sea_orm::DatabaseConnection>) -> Self {
    let delay_millis = Duration::from_millis(
      env::var("LOADER_DELAY_MILLIS")
        .unwrap_or_else(|_| "1".to_string())
        .parse::<u64>()
        .unwrap_or(1),
    );

    LoaderManager {
      db: db.clone(),
      cms_navigation_item_page: DataLoader::new(
        cms_navigation_items::Entity.to_entity_relation_loader(db.clone()),
        tokio::spawn,
      )
      .delay(delay_millis),
      cms_navigation_item_section: DataLoader::new(
        cms_navigation_items::Entity
          .to_entity_link_loader(CmsNavigationItemToCmsNavigationSection, db.clone()),
        tokio::spawn,
      )
      .delay(delay_millis),
      convention_event_categories: DataLoader::new(
        <conventions::Entity as ToEntityRelationLoader<
          event_categories::Entity,
          conventions::PrimaryKey,
        >>::to_entity_relation_loader(&conventions::Entity::default(), db.clone()),
        tokio::spawn,
      )
      .delay(delay_millis),
      convention_ticket_types: DataLoader::new(
        <conventions::Entity as ToEntityRelationLoader<
          ticket_types::Entity,
          conventions::PrimaryKey,
        >>::to_entity_relation_loader(&conventions::Entity::default(), db.clone()),
        tokio::spawn,
      )
      .delay(delay_millis),
      conventions_by_id: DataLoader::new(
        conventions::Entity.to_entity_id_loader(db.clone()),
        tokio::spawn,
      )
      .delay(delay_millis),
      order_order_entries: DataLoader::new(
        orders::Entity.to_entity_relation_loader(db.clone()),
        tokio::spawn,
      )
      .delay(delay_millis),
      staff_positions_by_id: DataLoader::new(
        staff_positions::Entity.to_entity_id_loader(db.clone()),
        tokio::spawn,
      )
      .delay(delay_millis),
      team_member_event: DataLoader::new(
        team_members::Entity.to_entity_relation_loader(db.clone()),
        tokio::spawn,
      )
      .delay(delay_millis),
      team_members_by_id: DataLoader::new(
        team_members::Entity.to_entity_id_loader(db.clone()),
        tokio::spawn,
      )
      .delay(delay_millis),
      ticket_type_providing_products: DataLoader::new(
        ticket_types::Entity.to_entity_relation_loader(db.clone()),
        tokio::spawn,
      )
      .delay(delay_millis),
      user_con_profile_signups: DataLoader::new(
        <user_con_profiles::Entity as ToEntityRelationLoader<
          signups::Entity,
          user_con_profiles::PrimaryKey,
        >>::to_entity_relation_loader(&user_con_profiles::Entity::default(), db.clone()),
        tokio::spawn,
      )
      .delay(delay_millis),
      user_con_profile_staff_positions: DataLoader::new(
        user_con_profiles::Entity.to_entity_link_loader(UserConProfileToStaffPositions, db.clone()),
        tokio::spawn,
      )
      .delay(delay_millis),
      user_con_profile_team_members: DataLoader::new(
        <user_con_profiles::Entity as ToEntityRelationLoader<
          team_members::Entity,
          user_con_profiles::PrimaryKey,
        >>::to_entity_relation_loader(&user_con_profiles::Entity::default(), db.clone()),
        tokio::spawn,
      )
      .delay(delay_millis),
      user_con_profile_ticket: DataLoader::new(
        <user_con_profiles::Entity as ToEntityRelationLoader<
          tickets::Entity,
          user_con_profiles::PrimaryKey,
        >>::to_entity_relation_loader(&user_con_profiles::Entity::default(), db.clone()),
        tokio::spawn,
      )
      .delay(delay_millis),
      user_con_profile_user: DataLoader::new(
        <user_con_profiles::Entity as ToEntityRelationLoader<
          users::Entity,
          user_con_profiles::PrimaryKey,
        >>::to_entity_relation_loader(&user_con_profiles::Entity::default(), db.clone()),
        tokio::spawn,
      )
      .delay(delay_millis),
      users_by_id: DataLoader::new(users::Entity.to_entity_id_loader(db.clone()), tokio::spawn)
        .delay(delay_millis),
    }
  }
}

impl std::fmt::Debug for LoaderManager {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    // DataLoader doesn't implement Debug, so we're just going to exclude the loaders from the debug output
    f.debug_struct("LoaderManager")
      .field("db", &self.db)
      .finish_non_exhaustive()
  }
}

impl Clone for LoaderManager {
  fn clone(&self) -> Self {
    LoaderManager::new(&self.db)
  }
}
