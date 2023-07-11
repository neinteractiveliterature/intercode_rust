use std::env;
use std::time::Duration;

use async_graphql::dataloader::DataLoader;

use intercode_entities::links::{
  CmsNavigationItemToCmsNavigationSection, ConventionToCatchAllStaffPosition, ConventionToSignups,
  ConventionToSingleEvent, ConventionToStaffPositions, EventCategoryToEventForm,
  EventCategoryToEventProposalForm, EventToProvidedTickets, FormToEventCategories, FormToFormItems,
  FormToProposalEventCategories, FormToUserConProfileConventions, SignupRequestToReplaceSignup,
  SignupRequestToResultSignup, StaffPositionToUserConProfiles, TicketToProvidedByEvent,
  UserActivityAlertToNotificationDestinations, UserConProfileToStaffPositions,
};
use intercode_entities::model_ext::FormResponse;
use intercode_entities::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use seawater::loaders::{EntityIdLoader, EntityLinkLoader, EntityRelationLoader};
use seawater::ConnectionWrapper;

use super::active_storage_attached_blobs_loader::ActiveStorageAttachedBlobsLoader;
use super::cms_content_group_contents_loader::CmsContentGroupContentsLoader;
use super::event_user_con_profile_event_rating_loader::EventUserConProfileEventRatingLoader;
use super::filtered_event_runs_loader::{EventRunsLoaderFilter, FilteredEventRunsLoader};
use super::loader_spawner::LoaderSpawner;
use super::order_quantity_by_status_loader::{
  OrderQuantityByStatusLoader, OrderQuantityByStatusLoaderEntity,
};
use super::permissioned_models_loader::PermissionedModelsLoader;
use super::permissioned_roles_loader::PermissionedRolesLoader;
use super::run_user_con_profile_signup_requests_loader::RunUserConProfileSignupRequestsLoader;
use super::run_user_con_profile_signups_loader::RunUserConProfileSignupsLoader;
use super::signup_count_loader::SignupCountLoader;
use super::waitlist_position_loader::WaitlistPositionLoader;

macro_rules! loader_manager {
  (@fields entity_id($name: ident, $entity: ident); $($tail:tt)*) => {
    loader_manager! {
      @fields $($tail)* $name: DataLoader<EntityIdLoader<$entity::Entity>>,
    }
  };

  (@fields entity_relation($name: ident, $from: ident, $to: ident); $($tail:tt)*) => {
    loader_manager! {
      @fields $($tail)* $name: DataLoader<EntityRelationLoader<$from::Entity, $to::Entity>>,
    }
  };

  (@fields entity_link($name: ident, $link: ident); $($tail:tt)*) => {
    loader_manager! {
      @fields $($tail)* $name: DataLoader<EntityLinkLoader<$link>>,
    }
  };

  (@fields $($tail:tt)*) => {
    pub struct LoaderManager {
      pub cms_content_group_contents: DataLoader<CmsContentGroupContentsLoader>,
      pub cms_file_file: DataLoader<ActiveStorageAttachedBlobsLoader>,
      pub convention_favicon: DataLoader<ActiveStorageAttachedBlobsLoader>,
      pub convention_open_graph_image: DataLoader<ActiveStorageAttachedBlobsLoader>,
      pub event_attached_images: DataLoader<ActiveStorageAttachedBlobsLoader>,
      pub event_proposal_attached_images: DataLoader<ActiveStorageAttachedBlobsLoader>,
      pub event_runs_filtered: LoaderSpawner<EventRunsLoaderFilter, i64, FilteredEventRunsLoader>,
      pub event_user_con_profile_event_ratings:
        LoaderSpawner<i64, i64, EventUserConProfileEventRatingLoader>,
      pub permissioned_models: DataLoader<PermissionedModelsLoader>,
      pub permissioned_roles: DataLoader<PermissionedRolesLoader>,
      pub product_image: DataLoader<ActiveStorageAttachedBlobsLoader>,
      pub product_order_quantity_by_status: DataLoader<OrderQuantityByStatusLoader>,
      pub product_variant_image: DataLoader<ActiveStorageAttachedBlobsLoader>,
      pub product_variant_order_quantity_by_status: DataLoader<OrderQuantityByStatusLoader>,
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
      @constructor $db, $delay_millis, $($tail)* $name: DataLoader::new(
        EntityIdLoader::<$entity::Entity>::new($db.clone(), $entity::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay($delay_millis),
    }
  };

  (@constructor $db: expr, $delay_millis: expr, entity_relation($name: ident, $from: ident, $to: ident); $($tail:tt)*) => {
    loader_manager! {
      @constructor $db, $delay_millis, $($tail)* $name: DataLoader::new(
        EntityRelationLoader::<$from::Entity, $to::Entity>::new($db.clone(), $from::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay($delay_millis),
    }
  };

  (@constructor $db: expr, $delay_millis: expr, entity_link($name: ident, $link: ident); $($tail:tt)*) => {
    loader_manager! {
      @constructor $db, $delay_millis, $($tail)* $name: DataLoader::new(
        EntityLinkLoader::<$link>::new(
          $db.clone(), $link,
          <<$link as sea_orm::Linked>::FromEntity as sea_orm::EntityTrait>::PrimaryKey::Id,
        ),
        tokio::spawn,
      )
      .delay($delay_millis),
    }
  };

  (@constructor $db: expr, $delay_millis: expr, $($tail:tt)*) => {
    LoaderManager {
      cms_content_group_contents: DataLoader::new(
        CmsContentGroupContentsLoader::new($db.clone(), $delay_millis),
        tokio::spawn,
      ).delay($delay_millis),
      cms_file_file: DataLoader::new(
        ActiveStorageAttachedBlobsLoader::new($db.clone(), active_storage_attachments::Entity::find()
          .filter(active_storage_attachments::Column::RecordType.eq("CmsFile"))
          .filter(active_storage_attachments::Column::Name.eq("file"))),
        tokio::spawn,
      ).delay($delay_millis),
      convention_favicon: DataLoader::new(
        ActiveStorageAttachedBlobsLoader::new($db.clone(), active_storage_attachments::Entity::find()
          .filter(active_storage_attachments::Column::RecordType.eq("Convention"))
          .filter(active_storage_attachments::Column::Name.eq("favicon"))),
        tokio::spawn,
      ).delay($delay_millis),
      convention_open_graph_image: DataLoader::new(
        ActiveStorageAttachedBlobsLoader::new($db.clone(), active_storage_attachments::Entity::find()
          .filter(active_storage_attachments::Column::RecordType.eq("Convention"))
          .filter(active_storage_attachments::Column::Name.eq("open_graph_image"))),
        tokio::spawn,
      ).delay($delay_millis),
      event_attached_images: DataLoader::new(
        ActiveStorageAttachedBlobsLoader::new($db.clone(), events::Model::attached_images_scope()),
        tokio::spawn,
      )
      .delay($delay_millis),
      event_proposal_attached_images: DataLoader::new(
        ActiveStorageAttachedBlobsLoader::new($db.clone(), event_proposals::Model::attached_images_scope()),
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
      permissioned_models: DataLoader::new(
        PermissionedModelsLoader::new($db.clone(), $delay_millis),
        tokio::spawn,
      )
      .delay($delay_millis),
      permissioned_roles: DataLoader::new(
        PermissionedRolesLoader::new($db.clone(), $delay_millis),
        tokio::spawn,
      )
      .delay($delay_millis),
      product_image: DataLoader::new(
        ActiveStorageAttachedBlobsLoader::new($db.clone(), active_storage_attachments::Entity::find()
          .filter(active_storage_attachments::Column::RecordType.eq("Product"))
          .filter(active_storage_attachments::Column::Name.eq("image"))),
        tokio::spawn,
      ).delay($delay_millis),
      product_order_quantity_by_status: DataLoader::new(
        OrderQuantityByStatusLoader::new($db.clone(), OrderQuantityByStatusLoaderEntity::Product),
        tokio::spawn,
      ).delay($delay_millis),
      product_variant_image: DataLoader::new(
        ActiveStorageAttachedBlobsLoader::new($db.clone(), active_storage_attachments::Entity::find()
          .filter(active_storage_attachments::Column::RecordType.eq("ProductVariant"))
          .filter(active_storage_attachments::Column::Name.eq("image"))),
        tokio::spawn,
      ).delay($delay_millis),
      product_variant_order_quantity_by_status: DataLoader::new(
        OrderQuantityByStatusLoader::new($db.clone(), OrderQuantityByStatusLoaderEntity::ProductVariant),
        tokio::spawn,
      ).delay($delay_millis),
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
      signup_waitlist_position: DataLoader::new(WaitlistPositionLoader::new($db.clone()), tokio::spawn)
        .delay($delay_millis),
      $($tail)*
    }
  };

  (@getters entity_id($name: ident, $entity: ident); $($tail:tt)*) => {
    loader_manager! {
      @getters $($tail)*

      pub fn $name(&self) -> &DataLoader<EntityIdLoader<$entity::Entity>> {
        &self.$name
      }
    }
  };

  (@getters entity_relation($name: ident, $from: ident, $to: ident); $($tail:tt)*) => {
    loader_manager! {
      @getters $($tail)*

      pub fn $name(&self) -> &DataLoader<EntityRelationLoader<$from::Entity, $to::Entity>> {
        &self.$name
      }
    }
  };

  (@getters entity_link($name: ident, $link: ident); $($tail:tt)*) => {
    loader_manager! {
      @getters $($tail)*

      pub fn $name(&self) -> &DataLoader<EntityLinkLoader<$link>> {
        &self.$name
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

      pub fn product_image(&self) -> &DataLoader<ActiveStorageAttachedBlobsLoader> {
        &self.product_image
      }

      pub fn product_variant_image(&self) -> &DataLoader<ActiveStorageAttachedBlobsLoader> {
        &self.product_variant_image
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
  entity_relation(cms_content_group_permissions, cms_content_groups, permissions);
  entity_relation(cms_navigation_item_page, cms_navigation_items, pages);
  entity_link(
    cms_navigation_item_section,
    CmsNavigationItemToCmsNavigationSection
  );
  entity_link(convention_catch_all_staff_position, ConventionToCatchAllStaffPosition);
  entity_relation(convention_departments, conventions, departments);
  entity_relation(convention_event_categories, conventions, event_categories);
  entity_relation(convention_notification_templates, conventions, notification_templates);
  entity_relation(convention_products, conventions, products);
  entity_relation(convention_rooms, conventions, rooms);
  entity_link(convention_signups, ConventionToSignups);
  entity_link(convention_single_event, ConventionToSingleEvent);
  entity_link(convention_staff_positions, ConventionToStaffPositions);
  entity_relation(convention_ticket_types, conventions, ticket_types);
  entity_relation(convention_user_activity_alerts, conventions, user_activity_alerts);
  entity_relation(convention_user_con_profile_form, conventions, forms);
  entity_id(conventions_by_id, conventions);
  entity_relation(coupon_application_coupon, coupon_applications, coupons);
  entity_relation(coupon_application_order, coupon_applications, orders);
  entity_relation(coupon_provides_product, coupons, products);
  entity_relation(department_event_categories, departments, event_categories);
  entity_relation(event_convention, events, conventions);
  entity_relation(event_event_category, events, event_categories);
  entity_relation(event_maximum_event_provided_tickets_overrides, events, maximum_event_provided_tickets_overrides);
  entity_link(event_provided_tickets, EventToProvidedTickets);
  entity_relation(event_runs, events, runs);
  entity_relation(event_team_members, events, team_members);
  entity_id(events_by_id, events);
  entity_relation(event_category_convention, event_categories, conventions);
  entity_relation(event_category_department, event_categories, departments);
  entity_link(event_category_event_form, EventCategoryToEventForm);
  entity_link(
    event_category_event_proposal_form,
    EventCategoryToEventProposalForm
  );
  entity_relation(event_proposal_convention, event_proposals, conventions);
  entity_relation(event_proposal_event, event_proposals, events);
  entity_relation(event_proposal_event_category, event_proposals, event_categories);
  entity_relation(event_proposal_owner, event_proposals, user_con_profiles);
  entity_id(event_proposals_by_id, event_proposals);
  entity_link(form_event_categories, FormToEventCategories);
  entity_link(form_form_items, FormToFormItems);
  entity_relation(form_form_sections, forms, form_sections);
  entity_link(form_proposal_event_categories, FormToProposalEventCategories);
  entity_link(form_user_con_profile_conventions, FormToUserConProfileConventions);
  entity_relation(form_section_form_items, form_sections, form_items);
  entity_relation(maximum_event_provided_tickets_override_event, maximum_event_provided_tickets_overrides, events);
  entity_relation(
    maximum_event_provided_tickets_override_ticket_type,
    maximum_event_provided_tickets_overrides,
    ticket_types
  );
  entity_relation(notification_destination_staff_position, notification_destinations, staff_positions);
  entity_relation(notification_destination_user_con_profile, notification_destinations, user_con_profiles);
  entity_relation(order_coupon_applications, orders, coupon_applications);
  entity_relation(order_order_entries, orders, order_entries);
  entity_relation(order_user_con_profile, orders, user_con_profiles);
  entity_relation(order_entry_order, order_entries, orders);
  entity_relation(order_entry_product, order_entries, products);
  entity_relation(order_entry_product_variant, order_entries, product_variants);
  entity_relation(organization_conventions, organizations, conventions);
  entity_relation(organization_organization_roles, organizations, organization_roles);
  entity_relation(organization_role_permissions, organization_roles, permissions);
  entity_relation(organization_role_users, organization_roles, users);
  entity_relation(pages_cms_layouts, pages, cms_layouts);
  entity_relation(product_product_variants, products, product_variants);
  entity_relation(product_provides_ticket_type, products, ticket_types);
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
  entity_relation(signup_request_user_con_profile, signup_requests, user_con_profiles);
  entity_id(staff_positions_by_id, staff_positions);
  entity_relation(staff_position_permissions, staff_positions, permissions);
  entity_link(staff_position_user_con_profiles, StaffPositionToUserConProfiles);
  entity_relation(team_member_event, team_members, events);
  entity_relation(
    team_member_user_con_profile,
    team_members,
    user_con_profiles
  );
  entity_id(team_members_by_id, team_members);
  entity_relation(ticket_order_entry, tickets, order_entries);
  entity_link(ticket_provided_by_event, TicketToProvidedByEvent);
  entity_relation(ticket_ticket_type, tickets, ticket_types);
  entity_relation(ticket_user_con_profile, tickets, user_con_profiles);
  entity_relation(ticket_type_providing_products, ticket_types, products);
  entity_link(user_activity_alert_notification_destinations, UserActivityAlertToNotificationDestinations);
  entity_relation(user_activity_alert_user, user_activity_alerts, users);
  entity_id(user_con_profiles_by_id, user_con_profiles);
  entity_relation(user_con_profile_convention, user_con_profiles, conventions);
  entity_relation(user_con_profile_orders, user_con_profiles, orders);
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
    f.debug_struct("LoaderManager").finish_non_exhaustive()
  }
}
