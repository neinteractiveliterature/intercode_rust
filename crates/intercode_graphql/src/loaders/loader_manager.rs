use std::any::Any;
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_graphql::dataloader::DataLoader;

use intercode_entities::links::{
  CmsNavigationItemToCmsNavigationSection, ConventionToStaffPositions, EventCategoryToEventForm,
  EventCategoryToEventProposalForm, EventToProvidedTickets, FormToFormItems,
  SignupRequestToReplaceSignup, SignupRequestToResultSignup, StaffPositionToUserConProfiles,
  TicketToProvidedByEvent, UserConProfileToStaffPositions,
};
use intercode_entities::model_ext::FormResponse;
use intercode_entities::*;
use seawater::loaders::{EntityIdLoader, EntityLinkLoader, EntityRelationLoader};
use seawater::ConnectionWrapper;

use super::active_storage_attached_blobs_loader::ActiveStorageAttachedBlobsLoader;
use super::event_user_con_profile_event_rating_loader::EventUserConProfileEventRatingLoader;
use super::filtered_event_runs_loader::{EventRunsLoaderFilter, FilteredEventRunsLoader};
use super::loader_spawner::LoaderSpawner;
use super::run_user_con_profile_signup_requests_loader::RunUserConProfileSignupRequestsLoader;
use super::run_user_con_profile_signups_loader::RunUserConProfileSignupsLoader;
use super::signup_count_loader::SignupCountLoader;
use super::waitlist_position_loader::WaitlistPositionLoader;

pub struct LoaderManager {
  db: ConnectionWrapper,
  delay: Duration,
  loaders_by_type: Arc<Mutex<HashMap<std::any::TypeId, Arc<dyn Any + Send + Sync>>>>,
  pub conventions_by_id: DataLoader<EntityIdLoader<conventions::Entity>>,
  pub events_by_id: DataLoader<EntityIdLoader<events::Entity>>,
  pub event_attached_images: DataLoader<ActiveStorageAttachedBlobsLoader>,
  pub event_runs_filtered: LoaderSpawner<EventRunsLoaderFilter, i64, FilteredEventRunsLoader>,
  pub event_user_con_profile_event_ratings:
    LoaderSpawner<i64, i64, EventUserConProfileEventRatingLoader>,
  pub run_signup_counts: DataLoader<SignupCountLoader>,
  pub run_user_con_profile_signups: LoaderSpawner<i64, i64, RunUserConProfileSignupsLoader>,
  pub run_user_con_profile_signup_requests:
    LoaderSpawner<i64, i64, RunUserConProfileSignupRequestsLoader>,
  pub runs_by_id: DataLoader<EntityIdLoader<runs::Entity>>,
  pub signup_waitlist_position: DataLoader<WaitlistPositionLoader>,
  pub staff_positions_by_id: DataLoader<EntityIdLoader<staff_positions::Entity>>,
  pub team_members_by_id: DataLoader<EntityIdLoader<team_members::Entity>>,
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
      delay: delay_millis,
      loaders_by_type: Arc::new(Mutex::new(Default::default())),
      conventions_by_id: DataLoader::new(
        EntityIdLoader::new(db.clone(), conventions::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay_millis),
      event_attached_images: DataLoader::new(
        ActiveStorageAttachedBlobsLoader::new(db.clone(), events::Model::attached_images_scope()),
        tokio::spawn,
      )
      .delay(delay_millis),
      events_by_id: DataLoader::new(
        EntityIdLoader::new(db.clone(), events::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay_millis),
      event_runs_filtered: LoaderSpawner::new(
        db.clone(),
        delay_millis,
        FilteredEventRunsLoader::new,
      ),
      event_user_con_profile_event_ratings: LoaderSpawner::new(
        db.clone(),
        delay_millis,
        EventUserConProfileEventRatingLoader::new,
      ),
      run_signup_counts: DataLoader::new(SignupCountLoader::new(db.clone()), tokio::spawn)
        .delay(delay_millis),
      run_user_con_profile_signups: LoaderSpawner::new(
        db.clone(),
        delay_millis,
        RunUserConProfileSignupsLoader::new,
      ),
      run_user_con_profile_signup_requests: LoaderSpawner::new(
        db.clone(),
        delay_millis,
        RunUserConProfileSignupRequestsLoader::new,
      ),
      runs_by_id: DataLoader::new(
        EntityIdLoader::new(db.clone(), runs::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay_millis),

      signup_waitlist_position: DataLoader::new(
        WaitlistPositionLoader::new(db.clone()),
        tokio::spawn,
      )
      .delay(delay_millis),

      staff_positions_by_id: DataLoader::new(
        EntityIdLoader::new(db.clone(), staff_positions::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay_millis),

      team_members_by_id: DataLoader::new(
        EntityIdLoader::new(db.clone(), team_members::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay_millis),

      users_by_id: DataLoader::new(EntityIdLoader::new(db, users::PrimaryKey::Id), tokio::spawn)
        .delay(delay_millis),
    }
  }
}

macro_rules! entity_relation_loader {
  ($name: ident, $from: ident, $to: ident) => {
    pub fn $name(&self) -> Arc<DataLoader<EntityRelationLoader<$from::Entity, $to::Entity>>> {
      let mut lock = self.loaders_by_type.lock().unwrap();

      let loader = lock
        .entry(std::any::TypeId::of::<
          EntityRelationLoader<$from::Entity, $to::Entity>,
        >())
        .or_insert_with(|| {
          Arc::new(
            DataLoader::new(
              EntityRelationLoader::<$from::Entity, $to::Entity>::new(
                self.db.clone(),
                $from::PrimaryKey::Id,
              ),
              tokio::spawn,
            )
            .delay(self.delay),
          )
        });

      loader
        .clone()
        .downcast::<DataLoader<EntityRelationLoader<$from::Entity, $to::Entity>>>()
        .unwrap()
    }
  };
}

macro_rules! entity_link_loader {
  ($name: ident, $link: ident) => {
    pub fn $name(&self) -> Arc<DataLoader<EntityLinkLoader<$link>>> {
      let mut lock = self.loaders_by_type.lock().unwrap();

      let loader = lock
        .entry(std::any::TypeId::of::<EntityLinkLoader<$link>>())
        .or_insert_with(|| {
          Arc::new(
            DataLoader::new(
              EntityLinkLoader::<$link>::new(
                self.db.clone(),
                $link,
                <<$link as sea_orm::Linked>::FromEntity as sea_orm::EntityTrait>::PrimaryKey::Id,
              ),
              tokio::spawn,
            )
            .delay(self.delay),
          )
        });

      loader
        .clone()
        .downcast::<DataLoader<EntityLinkLoader<$link>>>()
        .unwrap()
    }
  };
}

impl LoaderManager {
  entity_relation_loader!(cms_navigation_item_page, cms_navigation_items, pages);
  entity_link_loader!(
    cms_navigation_item_section,
    CmsNavigationItemToCmsNavigationSection
  );
  entity_relation_loader!(convention_event_categories, conventions, event_categories);
  entity_relation_loader!(convention_rooms, conventions, rooms);
  entity_link_loader!(convention_staff_positions, ConventionToStaffPositions);
  entity_relation_loader!(convention_ticket_types, conventions, ticket_types);
  entity_relation_loader!(event_event_category, events, event_categories);
  entity_link_loader!(event_provided_tickets, EventToProvidedTickets);
  entity_relation_loader!(event_runs, events, runs);
  entity_relation_loader!(event_team_members, events, team_members);
  entity_link_loader!(event_category_event_form, EventCategoryToEventForm);
  entity_link_loader!(
    event_category_event_proposal_form,
    EventCategoryToEventProposalForm
  );
  entity_link_loader!(form_form_items, FormToFormItems);
  entity_relation_loader!(form_form_sections, forms, form_sections);
  entity_relation_loader!(form_section_form_items, form_sections, form_items);
  entity_relation_loader!(order_order_entries, orders, order_entries);
  entity_relation_loader!(room_runs, rooms, runs);
  entity_relation_loader!(run_event, runs, events);
  entity_relation_loader!(run_rooms, runs, rooms);
  entity_relation_loader!(run_signups, runs, signups);
  entity_relation_loader!(signup_run, signups, runs);
  entity_relation_loader!(signup_user_con_profile, signups, user_con_profiles);
  entity_link_loader!(signup_request_replace_signup, SignupRequestToReplaceSignup);
  entity_link_loader!(signup_request_result_signup, SignupRequestToResultSignup);
  entity_relation_loader!(signup_request_target_run, signup_requests, runs);
  entity_link_loader!(
    staff_position_user_con_profiles,
    StaffPositionToUserConProfiles
  );
  entity_relation_loader!(team_member_event, team_members, events);
  entity_relation_loader!(
    team_member_user_con_profile,
    team_members,
    user_con_profiles
  );
  entity_link_loader!(ticket_provided_by_event, TicketToProvidedByEvent);
  entity_relation_loader!(ticket_user_con_profile, tickets, user_con_profiles);
  entity_relation_loader!(ticket_type_providing_products, ticket_types, products);
  entity_relation_loader!(user_con_profile_signups, user_con_profiles, signups);
  entity_link_loader!(
    user_con_profile_staff_positions,
    UserConProfileToStaffPositions
  );
  entity_relation_loader!(
    user_con_profile_team_members,
    user_con_profiles,
    team_members
  );
  entity_relation_loader!(user_con_profile_ticket, user_con_profiles, tickets);
  entity_relation_loader!(user_con_profile_user, user_con_profiles, users);
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
