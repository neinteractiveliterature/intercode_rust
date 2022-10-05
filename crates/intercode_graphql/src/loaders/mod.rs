#[macro_use]
mod entities_by_id_loader;
#[macro_use]
mod entities_by_relation_loader;
#[macro_use]
mod entities_by_link_loader;
mod cms_navigation_item_loaders;
mod convention_loaders;
mod event_loaders;
pub mod expect;
mod order_loaders;
mod run_loaders;
mod staff_position_loaders;
mod team_member_loaders;
mod ticket_type_loaders;
mod user_con_profile_loaders;
mod user_loaders;

use std::env;
use std::sync::Arc;
use std::time::Duration;

use async_graphql::dataloader::DataLoader;
pub use entities_by_id_loader::*;
pub use entities_by_link_loader::*;
pub use entities_by_relation_loader::*;

use intercode_entities::links::{ConventionToStaffPositions, StaffPositionToUserConProfiles};
use intercode_entities::*;

use self::cms_navigation_item_loaders::CmsNavigationItemToCmsNavigationSection;
use self::run_loaders::RunToRooms;
use self::user_con_profile_loaders::UserConProfileToStaffPositions;

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
  pub convention_staff_positions: DataLoader<
    EntityLinkLoader<
      conventions::Entity,
      ConventionToStaffPositions,
      staff_positions::Entity,
      conventions::PrimaryKey,
    >,
  >,
  pub convention_ticket_types: DataLoader<
    EntityRelationLoader<conventions::Entity, ticket_types::Entity, conventions::PrimaryKey>,
  >,
  pub conventions_by_id: DataLoader<EntityIdLoader<conventions::Entity, conventions::PrimaryKey>>,
  pub event_event_category:
    DataLoader<EntityRelationLoader<events::Entity, event_categories::Entity, events::PrimaryKey>>,
  pub event_runs:
    DataLoader<EntityRelationLoader<events::Entity, runs::Entity, events::PrimaryKey>>,
  pub event_team_members:
    DataLoader<EntityRelationLoader<events::Entity, team_members::Entity, events::PrimaryKey>>,
  pub events_by_id: DataLoader<EntityIdLoader<events::Entity, events::PrimaryKey>>,
  pub order_order_entries:
    DataLoader<EntityRelationLoader<orders::Entity, order_entries::Entity, orders::PrimaryKey>>,
  pub run_event: DataLoader<EntityRelationLoader<runs::Entity, events::Entity, runs::PrimaryKey>>,
  pub run_rooms:
    DataLoader<EntityLinkLoader<runs::Entity, RunToRooms, rooms::Entity, runs::PrimaryKey>>,
  pub staff_position_user_con_profiles: DataLoader<
    EntityLinkLoader<
      staff_positions::Entity,
      StaffPositionToUserConProfiles,
      user_con_profiles::Entity,
      staff_positions::PrimaryKey,
    >,
  >,
  pub staff_positions_by_id:
    DataLoader<EntityIdLoader<staff_positions::Entity, staff_positions::PrimaryKey>>,
  pub team_member_event: DataLoader<
    EntityRelationLoader<team_members::Entity, events::Entity, team_members::PrimaryKey>,
  >,
  pub team_member_user_con_profile: DataLoader<
    EntityRelationLoader<team_members::Entity, user_con_profiles::Entity, team_members::PrimaryKey>,
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
      convention_staff_positions: DataLoader::new(
        conventions::Entity.to_entity_link_loader(ConventionToStaffPositions, db.clone()),
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
      event_event_category: DataLoader::new(
        <events::Entity as ToEntityRelationLoader<event_categories::Entity, events::PrimaryKey>>::to_entity_relation_loader(&events::Entity, db.clone()),
        tokio::spawn,
      ).delay(delay_millis),
      event_runs: DataLoader::new(
        <events::Entity as ToEntityRelationLoader<runs::Entity, events::PrimaryKey>>::to_entity_relation_loader(&events::Entity, db.clone()),
        tokio::spawn,
      )
      .delay(delay_millis),
      event_team_members: DataLoader::new(
        <events::Entity as ToEntityRelationLoader<team_members::Entity, events::PrimaryKey>>::to_entity_relation_loader(&events::Entity, db.clone()),
        tokio::spawn,
      )
      .delay(delay_millis),
      events_by_id: DataLoader::new(events::Entity.to_entity_id_loader(db.clone()), tokio::spawn)
        .delay(delay_millis),
      order_order_entries: DataLoader::new(
        orders::Entity.to_entity_relation_loader(db.clone()),
        tokio::spawn,
      )
      .delay(delay_millis),
      run_event: DataLoader::new(
        runs::Entity.to_entity_relation_loader(db.clone()), tokio::spawn
      ).delay(delay_millis),
      run_rooms: DataLoader::new(
        runs::Entity.to_entity_link_loader(RunToRooms, db.clone()), tokio::spawn
      ).delay(delay_millis),
      staff_position_user_con_profiles: DataLoader::new(
        staff_positions::Entity.to_entity_link_loader(StaffPositionToUserConProfiles, db.clone()),
        tokio::spawn,
      )
      .delay(delay_millis),
      staff_positions_by_id: DataLoader::new(
        staff_positions::Entity.to_entity_id_loader(db.clone()),
        tokio::spawn,
      )
      .delay(delay_millis),
      team_member_user_con_profile: DataLoader::new(
        <team_members::Entity as ToEntityRelationLoader<user_con_profiles::Entity, team_members::PrimaryKey>>::to_entity_relation_loader(&team_members::Entity, db.clone()),
        tokio::spawn
      ).delay(delay_millis),
      team_member_event: DataLoader::new(
        <team_members::Entity as ToEntityRelationLoader<events::Entity, team_members::PrimaryKey>>::to_entity_relation_loader(&team_members::Entity, db.clone()),
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
