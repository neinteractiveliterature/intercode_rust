use std::env;
use std::time::Duration;

use async_graphql::dataloader::DataLoader;

use intercode_entities::links::{
  CmsNavigationItemToCmsNavigationSection, ConventionToStaffPositions,
  StaffPositionToUserConProfiles, UserConProfileToStaffPositions,
};
use intercode_entities::*;
use seawater::loaders::{EntityIdLoader, EntityLinkLoader, EntityRelationLoader};
use seawater::ConnectionWrapper;

pub struct LoaderManager {
  db: ConnectionWrapper,
  pub cms_navigation_item_page:
    DataLoader<EntityRelationLoader<cms_navigation_items::Entity, pages::Entity>>,
  pub cms_navigation_item_section: DataLoader<
    EntityLinkLoader<
      cms_navigation_items::Entity,
      CmsNavigationItemToCmsNavigationSection,
      cms_navigation_items::Entity,
    >,
  >,
  pub convention_event_categories:
    DataLoader<EntityRelationLoader<conventions::Entity, event_categories::Entity>>,
  pub convention_rooms: DataLoader<EntityRelationLoader<conventions::Entity, rooms::Entity>>,
  pub convention_staff_positions: DataLoader<
    EntityLinkLoader<conventions::Entity, ConventionToStaffPositions, staff_positions::Entity>,
  >,
  pub convention_ticket_types:
    DataLoader<EntityRelationLoader<conventions::Entity, ticket_types::Entity>>,
  pub conventions_by_id: DataLoader<EntityIdLoader<conventions::Entity>>,
  pub event_event_category:
    DataLoader<EntityRelationLoader<events::Entity, event_categories::Entity>>,
  pub event_runs: DataLoader<EntityRelationLoader<events::Entity, runs::Entity>>,
  pub event_team_members: DataLoader<EntityRelationLoader<events::Entity, team_members::Entity>>,
  pub events_by_id: DataLoader<EntityIdLoader<events::Entity>>,
  pub order_order_entries: DataLoader<EntityRelationLoader<orders::Entity, order_entries::Entity>>,
  pub room_runs: DataLoader<EntityRelationLoader<rooms::Entity, runs::Entity>>,
  pub run_event: DataLoader<EntityRelationLoader<runs::Entity, events::Entity>>,
  pub run_rooms: DataLoader<EntityRelationLoader<runs::Entity, rooms::Entity>>,
  pub staff_position_user_con_profiles: DataLoader<
    EntityLinkLoader<
      staff_positions::Entity,
      StaffPositionToUserConProfiles,
      user_con_profiles::Entity,
    >,
  >,
  pub staff_positions_by_id: DataLoader<EntityIdLoader<staff_positions::Entity>>,
  pub team_member_event: DataLoader<EntityRelationLoader<team_members::Entity, events::Entity>>,
  pub team_member_user_con_profile:
    DataLoader<EntityRelationLoader<team_members::Entity, user_con_profiles::Entity>>,
  pub team_members_by_id: DataLoader<EntityIdLoader<team_members::Entity>>,
  pub ticket_type_providing_products:
    DataLoader<EntityRelationLoader<ticket_types::Entity, products::Entity>>,
  pub user_con_profile_signups:
    DataLoader<EntityRelationLoader<user_con_profiles::Entity, signups::Entity>>,
  pub user_con_profile_staff_positions: DataLoader<
    EntityLinkLoader<
      user_con_profiles::Entity,
      UserConProfileToStaffPositions,
      staff_positions::Entity,
    >,
  >,
  pub user_con_profile_ticket:
    DataLoader<EntityRelationLoader<user_con_profiles::Entity, tickets::Entity>>,
  pub user_con_profile_user:
    DataLoader<EntityRelationLoader<user_con_profiles::Entity, users::Entity>>,
  pub user_con_profile_team_members:
    DataLoader<EntityRelationLoader<user_con_profiles::Entity, team_members::Entity>>,
  pub users_by_id: DataLoader<EntityIdLoader<users::Entity>>,
}

impl LoaderManager {
  pub fn new(db: ConnectionWrapper) -> Self {
    let delay_millis = Duration::from_millis(
      env::var("LOADER_DELAY_MILLIS")
        .unwrap_or_else(|_| "1".to_string())
        .parse::<u64>()
        .unwrap_or(1),
    );

    LoaderManager {
      db: db.clone(),
      cms_navigation_item_page: DataLoader::new(
        EntityRelationLoader::new(db.clone(), cms_navigation_items::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay_millis),
      cms_navigation_item_section: DataLoader::new(
        EntityLinkLoader::new(
          db.clone(),
          CmsNavigationItemToCmsNavigationSection,
          cms_navigation_items::PrimaryKey::Id,
        ),
        tokio::spawn,
      )
      .delay(delay_millis),
      convention_event_categories: DataLoader::new(
        EntityRelationLoader::new(db.clone(), conventions::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay_millis),
      convention_staff_positions: DataLoader::new(
        EntityLinkLoader::new(
          db.clone(),
          ConventionToStaffPositions,
          conventions::PrimaryKey::Id,
        ),
        tokio::spawn,
      )
      .delay(delay_millis),
      convention_rooms: DataLoader::new(
        EntityRelationLoader::new(db.clone(), conventions::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay_millis),
      convention_ticket_types: DataLoader::new(
        EntityRelationLoader::new(db.clone(), conventions::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay_millis),
      conventions_by_id: DataLoader::new(
        EntityIdLoader::new(db.clone(), conventions::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay_millis),
      event_event_category: DataLoader::new(
        EntityRelationLoader::new(db.clone(), events::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay_millis),
      event_runs: DataLoader::new(
        EntityRelationLoader::new(db.clone(), events::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay_millis),
      event_team_members: DataLoader::new(
        EntityRelationLoader::new(db.clone(), events::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay_millis),
      events_by_id: DataLoader::new(
        EntityIdLoader::new(db.clone(), events::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay_millis),
      order_order_entries: DataLoader::new(
        EntityRelationLoader::new(db.clone(), orders::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay_millis),
      room_runs: DataLoader::new(
        EntityRelationLoader::new(db.clone(), rooms::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay_millis),
      run_event: DataLoader::new(
        EntityRelationLoader::new(db.clone(), runs::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay_millis),
      run_rooms: DataLoader::new(
        EntityRelationLoader::new(db.clone(), runs::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay_millis),
      staff_position_user_con_profiles: DataLoader::new(
        EntityLinkLoader::new(
          db.clone(),
          StaffPositionToUserConProfiles,
          staff_positions::PrimaryKey::Id,
        ),
        tokio::spawn,
      )
      .delay(delay_millis),
      staff_positions_by_id: DataLoader::new(
        EntityIdLoader::new(db.clone(), staff_positions::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay_millis),
      team_member_user_con_profile: DataLoader::new(
        EntityRelationLoader::new(db.clone(), team_members::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay_millis),
      team_member_event: DataLoader::new(
        EntityRelationLoader::new(db.clone(), team_members::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay_millis),
      team_members_by_id: DataLoader::new(
        EntityIdLoader::new(db.clone(), team_members::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay_millis),
      ticket_type_providing_products: DataLoader::new(
        EntityRelationLoader::new(db.clone(), ticket_types::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay_millis),
      user_con_profile_signups: DataLoader::new(
        EntityRelationLoader::new(db.clone(), user_con_profiles::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay_millis),
      user_con_profile_staff_positions: DataLoader::new(
        EntityLinkLoader::new(
          db.clone(),
          UserConProfileToStaffPositions,
          user_con_profiles::PrimaryKey::Id,
        ),
        tokio::spawn,
      )
      .delay(delay_millis),
      user_con_profile_team_members: DataLoader::new(
        EntityRelationLoader::new(db.clone(), user_con_profiles::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay_millis),
      user_con_profile_ticket: DataLoader::new(
        EntityRelationLoader::new(db.clone(), user_con_profiles::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay_millis),
      user_con_profile_user: DataLoader::new(
        EntityRelationLoader::new(db.clone(), user_con_profiles::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay_millis),
      users_by_id: DataLoader::new(EntityIdLoader::new(db, users::PrimaryKey::Id), tokio::spawn)
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
    LoaderManager::new(self.db.clone())
  }
}
