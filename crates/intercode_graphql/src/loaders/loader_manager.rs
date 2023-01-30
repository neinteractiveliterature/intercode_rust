use std::env;
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
use once_cell::race::OnceBox;
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

macro_rules! loader_manager {
  (@fields entity_id($name: ident, $entity: ident); $($tail:tt)*) => {
    loader_manager! {
      @fields $($tail)* $name: OnceBox<DataLoader<EntityIdLoader<$entity::Entity>>>,
    }
  };

  (@fields entity_relation($name: ident, $from: ident, $to: ident); $($tail:tt)*) => {
    loader_manager! {
      @fields $($tail)* $name: OnceBox<DataLoader<EntityRelationLoader<$from::Entity, $to::Entity>>>,
    }
  };

  (@fields entity_link($name: ident, $link: ident); $($tail:tt)*) => {
    loader_manager! {
      @fields $($tail)* $name: OnceBox<DataLoader<EntityLinkLoader<$link>>>,
    }
  };

  (@fields $($tail:tt)*) => {
    pub struct LoaderManager {
      db: ConnectionWrapper,
      delay: Duration,
      pub event_attached_images: DataLoader<ActiveStorageAttachedBlobsLoader>,
      pub event_runs_filtered: LoaderSpawner<EventRunsLoaderFilter, i64, FilteredEventRunsLoader>,
      pub event_user_con_profile_event_ratings:
        LoaderSpawner<i64, i64, EventUserConProfileEventRatingLoader>,
      pub run_signup_counts: DataLoader<SignupCountLoader>,
      pub run_user_con_profile_signups: LoaderSpawner<i64, i64, RunUserConProfileSignupsLoader>,
      pub run_user_con_profile_signup_requests:
        LoaderSpawner<i64, i64, RunUserConProfileSignupRequestsLoader>,
      pub signup_waitlist_position: DataLoader<WaitlistPositionLoader>,
      $($tail)*
    }
  };

  (@constructor $db: expr, $delay_millis: expr, entity_id($name: ident, $entity: ident); $($tail:tt)*) => {
    loader_manager! {
      @constructor $db, $delay_millis, $($tail)* $name: Default::default(),
    }
  };

  (@constructor $db: expr, $delay_millis: expr, entity_relation($name: ident, $from: ident, $to: ident); $($tail:tt)*) => {
    loader_manager! {
      @constructor $db, $delay_millis, $($tail)* $name: Default::default(),
    }
  };

  (@constructor $db: expr, $delay_millis: expr, entity_link($name: ident, $link: ident); $($tail:tt)*) => {
    loader_manager! {
      @constructor $db, $delay_millis, $($tail)* $name: Default::default(),
    }
  };

  (@constructor $db: expr, $delay_millis: expr, $($tail:tt)*) => {
    LoaderManager {
      db: $db.clone(),
      delay: $delay_millis,
      event_attached_images: DataLoader::new(
        ActiveStorageAttachedBlobsLoader::new($db.clone(), events::Model::attached_images_scope()),
        tokio::spawn,
      )
      .delay($delay_millis),
      event_runs_filtered: LoaderSpawner::new(
        $db.clone(),
        $delay_millis,
        FilteredEventRunsLoader::new,
      ),
      event_user_con_profile_event_ratings: LoaderSpawner::new(
        $db.clone(),
        $delay_millis,
        EventUserConProfileEventRatingLoader::new,
      ),
      run_signup_counts: DataLoader::new(SignupCountLoader::new($db.clone()), tokio::spawn)
        .delay($delay_millis),
      run_user_con_profile_signups: LoaderSpawner::new(
        $db.clone(),
        $delay_millis,
        RunUserConProfileSignupsLoader::new,
      ),
      run_user_con_profile_signup_requests: LoaderSpawner::new(
        $db.clone(),
        $delay_millis,
        RunUserConProfileSignupRequestsLoader::new,
      ),
      signup_waitlist_position: DataLoader::new(WaitlistPositionLoader::new($db), tokio::spawn)
        .delay($delay_millis),
      $($tail)*
    }
  };

  (@getters entity_id($name: ident, $entity: ident); $($tail:tt)*) => {
    loader_manager! {
      @getters $($tail)*

      pub fn $name(&self) -> &DataLoader<EntityIdLoader<$entity::Entity>> {
        self.$name.get_or_init(||
          Box::new(
            DataLoader::new(
              EntityIdLoader::<$entity::Entity>::new(self.db.clone(), $entity::PrimaryKey::Id),
              tokio::spawn,
            )
            .delay(self.delay),
          )
        )
      }
    }
  };

  (@getters entity_relation($name: ident, $from: ident, $to: ident); $($tail:tt)*) => {
    loader_manager! {
      @getters $($tail)*

      pub fn $name(&self) -> &DataLoader<EntityRelationLoader<$from::Entity, $to::Entity>> {
        self.$name.get_or_init(||
          Box::new(
            DataLoader::new(
              EntityRelationLoader::<$from::Entity, $to::Entity>::new(self.db.clone(), $from::PrimaryKey::Id),
              tokio::spawn,
            )
            .delay(self.delay),
          )
        )
      }
    }
  };

  (@getters entity_link($name: ident, $link: ident); $($tail:tt)*) => {
    loader_manager! {
      @getters $($tail)*

      pub fn $name(&self) -> &DataLoader<EntityLinkLoader<$link>> {
        self.$name.get_or_init(||
          Box::new(
            DataLoader::new(
              EntityLinkLoader::<$link>::new(
                self.db.clone(), $link,
                <<$link as sea_orm::Linked>::FromEntity as sea_orm::EntityTrait>::PrimaryKey::Id,
              ),
              tokio::spawn,
            )
            .delay(self.delay),
          )
        )
      }
    }
  };

  (@getters $($tail:tt)*) => {
    $($tail)*
  };

  (@implementation $($tail:tt)*) => {
    impl LoaderManager {
      pub fn new(db: ConnectionWrapper) -> Self {
        let delay_millis = Duration::from_millis(
          env::var("LOADER_DELAY_MILLIS")
            .unwrap_or_else(|_| "1".to_string())
            .parse::<u64>()
            .unwrap_or(1),
        );

        loader_manager! {
          @constructor db, delay_millis, $($tail)*
        }
      }

      loader_manager! {
        @getters $($tail)*
      }
    }
  };

  ($($tail:tt)*) => {
    loader_manager! { @fields $($tail)* }
    loader_manager! { @implementation $($tail)* }
  };
}

loader_manager!(
  entity_relation(cms_navigation_item_page, cms_navigation_items, pages);
  entity_link(
    cms_navigation_item_section,
    CmsNavigationItemToCmsNavigationSection
  );
  entity_relation(convention_event_categories, conventions, event_categories);
  entity_relation(convention_rooms, conventions, rooms);
  entity_link(convention_staff_positions, ConventionToStaffPositions);
  entity_relation(convention_ticket_types, conventions, ticket_types);
  entity_id(conventions_by_id, conventions);
  entity_relation(event_event_category, events, event_categories);
  entity_link(event_provided_tickets, EventToProvidedTickets);
  entity_relation(event_runs, events, runs);
  entity_relation(event_team_members, events, team_members);
  entity_id(events_by_id, events);
  entity_link(event_category_event_form, EventCategoryToEventForm);
  entity_link(
    event_category_event_proposal_form,
    EventCategoryToEventProposalForm
  );
  entity_link(form_form_items, FormToFormItems);
  entity_relation(form_form_sections, forms, form_sections);
  entity_relation(form_section_form_items, form_sections, form_items);
  entity_relation(order_order_entries, orders, order_entries);
  entity_relation(room_runs, rooms, runs);
  entity_relation(run_event, runs, events);
  entity_relation(run_rooms, runs, rooms);
  entity_relation(run_signups, runs, signups);
  entity_id(runs_by_id, runs);
  entity_relation(signup_run, signups, runs);
  entity_relation(signup_user_con_profile, signups, user_con_profiles);
  entity_link(signup_request_replace_signup, SignupRequestToReplaceSignup);
  entity_link(signup_request_result_signup, SignupRequestToResultSignup);
  entity_relation(signup_request_target_run, signup_requests, runs);
  entity_link(
    staff_position_user_con_profiles,
    StaffPositionToUserConProfiles
  );
  entity_id(staff_positions_by_id, staff_positions);
  entity_relation(team_member_event, team_members, events);
  entity_relation(
    team_member_user_con_profile,
    team_members,
    user_con_profiles
  );
  entity_id(team_members_by_id, team_members);
  entity_link(ticket_provided_by_event, TicketToProvidedByEvent);
  entity_relation(ticket_user_con_profile, tickets, user_con_profiles);
  entity_relation(ticket_type_providing_products, ticket_types, products);
  entity_relation(user_con_profile_signups, user_con_profiles, signups);
  entity_link(
    user_con_profile_staff_positions,
    UserConProfileToStaffPositions
  );
  entity_relation(
    user_con_profile_team_members,
    user_con_profiles,
    team_members
  );
  entity_relation(user_con_profile_ticket, user_con_profiles, tickets);
  entity_relation(user_con_profile_user, user_con_profiles, users);
  entity_id(users_by_id, users);
);

impl std::fmt::Debug for LoaderManager {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    // DataLoader doesn't implement Debug, so we're just going to exclude the loaders from the debug output
    f.debug_struct("LoaderManager")
      .field("db", &self.db)
      .finish_non_exhaustive()
  }
}
