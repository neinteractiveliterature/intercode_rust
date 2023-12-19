use crate::{
  api::merged_objects::{
    DepartmentType, EventCategoryType, EventProposalType, EventType, FormType, MailingListsType,
    OrderType, SignupRequestType, SignupType, StaffPositionType, UserConProfileType,
  },
  merged_model_backed_type,
};

use super::{
  coupon_type::CouponType, product_type::ProductType, room_type::RoomType,
  ticket_type_type::TicketTypeType, user_activity_alert_type::UserActivityAlertType,
  CmsContentGroupType, OrganizationType, RunType,
};
use async_graphql::*;
use intercode_cms::{api::partial_objects::ConventionCmsFields, CmsParentImplementation};
use intercode_conventions::partial_objects::{
  ConventionConventionsExtensions, ConventionConventionsFields,
};
use intercode_entities::conventions;
use intercode_events::{
  partial_objects::{ConventionEventsExtensions, ConventionEventsFields},
  query_builders::{EventFiltersInput, EventProposalFiltersInput},
};
use intercode_forms::partial_objects::ConventionFormsExtensions;
use intercode_graphql_core::{
  model_backed_type, scalars::DateScalar, ModelBackedType, ModelPaginator,
};
use intercode_notifiers::partial_objects::ConventionNotifiersFields;
use intercode_query_builders::sort_input::SortInput;
use intercode_signups::{
  partial_objects::{ConventionSignupsExtensions, ConventionSignupsFields},
  query_builders::SignupRequestFiltersInput,
};
use intercode_store::{
  partial_objects::{ConventionStoreExtensions, ConventionStoreFields},
  query_builders::{CouponFiltersInput, OrderFiltersInput},
};
use intercode_users::{
  partial_objects::ConventionUsersExtensions, query_builders::UserConProfileFiltersInput,
};

model_backed_type!(ConventionGlueFields, conventions::Model);

impl CmsParentImplementation<conventions::Model> for ConventionGlueFields {}
impl ConventionConventionsExtensions for ConventionGlueFields {}
impl ConventionEventsExtensions for ConventionGlueFields {}
impl ConventionFormsExtensions for ConventionGlueFields {}
impl ConventionSignupsExtensions for ConventionGlueFields {}
impl ConventionStoreExtensions for ConventionGlueFields {}
impl ConventionUsersExtensions for ConventionGlueFields {}

#[Object]
impl ConventionGlueFields {
  #[graphql(name = "bio_eligible_user_con_profiles")]
  pub async fn bio_eligible_user_con_profiles(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<UserConProfileType>, Error> {
    ConventionUsersExtensions::bio_eligible_user_con_profiles(self, ctx).await
  }

  #[graphql(name = "catch_all_staff_position")]
  pub async fn catch_all_staff_position(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<StaffPositionType>> {
    ConventionUsersExtensions::catch_all_staff_position(self, ctx).await
  }

  async fn cms_content_groups(&self, ctx: &Context<'_>) -> Result<Vec<CmsContentGroupType>, Error> {
    CmsParentImplementation::cms_content_groups(self, ctx).await
  }

  async fn cms_content_group(
    &self,
    ctx: &Context<'_>,
    id: ID,
  ) -> Result<CmsContentGroupType, Error> {
    CmsParentImplementation::cms_content_group(self, ctx, id).await
  }

  #[graphql(name = "coupons_paginated")]
  async fn coupons_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    #[graphql(name = "per_page")] per_page: Option<u64>,
    filters: Option<CouponFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<ModelPaginator<CouponType>, Error> {
    ConventionStoreExtensions::coupons_paginated(self, ctx, page, per_page, filters, sort).await
  }

  async fn departments(&self, ctx: &Context<'_>) -> Result<Vec<DepartmentType>> {
    ConventionConventionsExtensions::departments(self, ctx).await
  }

  /// Finds an active event by ID in this convention. If there is no event with that ID in this
  /// convention, or the event is no longer active, errors out.
  pub async fn event(&self, ctx: &Context<'_>, id: Option<ID>) -> Result<EventType, Error> {
    ConventionEventsExtensions::event(self, ctx, id).await
  }

  pub async fn events(
    &self,
    ctx: &Context<'_>,
    start: Option<DateScalar>,
    finish: Option<DateScalar>,
    include_dropped: Option<bool>,
    filters: Option<EventFiltersInput>,
  ) -> Result<Vec<EventType>, Error> {
    ConventionEventsExtensions::events(self, ctx, start, finish, include_dropped, filters).await
  }

  #[graphql(name = "events_paginated")]
  pub async fn events_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    #[graphql(name = "per_page")] per_page: Option<u64>,
    filters: Option<EventFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<ModelPaginator<EventType>, Error> {
    ConventionEventsExtensions::events_paginated(self, ctx, page, per_page, filters, sort).await
  }

  #[graphql(name = "event_categories")]
  async fn event_categories(
    &self,
    ctx: &Context<'_>,
    #[graphql(name = "current_ability_can_read_event_proposals")]
    current_ability_can_read_event_proposals: Option<bool>,
  ) -> Result<Vec<EventCategoryType>, Error> {
    ConventionEventsExtensions::event_categories(
      self,
      ctx,
      current_ability_can_read_event_proposals,
    )
    .await
  }

  /// Finds an event proposal by ID in this convention. If there is no event proposal with that ID
  /// in this convention, errors out.
  #[graphql(name = "event_proposal")]
  async fn event_proposal(
    &self,
    ctx: &Context<'_>,
    #[graphql(desc = "The ID of the event proposal to find.")] id: Option<ID>,
  ) -> Result<EventProposalType> {
    ConventionEventsExtensions::event_proposal(self, ctx, id).await
  }

  #[graphql(name = "event_proposals_paginated")]
  pub async fn event_proposals_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    #[graphql(name = "per_page")] per_page: Option<u64>,
    filters: Option<EventProposalFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<ModelPaginator<EventProposalType>, Error> {
    ConventionEventsExtensions::event_proposals_paginated(self, ctx, page, per_page, filters, sort)
      .await
  }

  pub async fn form(&self, ctx: &Context<'_>, id: Option<ID>) -> Result<FormType> {
    ConventionFormsExtensions::form(self, ctx, id).await
  }

  pub async fn forms(&self, ctx: &Context<'_>) -> Result<Vec<FormType>> {
    ConventionFormsExtensions::forms(self, ctx).await
  }

  #[graphql(name = "mailing_lists")]
  async fn mailing_lists(&self) -> MailingListsType {
    MailingListsType::from_type(self.clone())
  }

  #[graphql(name = "my_profile")]
  pub async fn my_profile(&self, ctx: &Context<'_>) -> Result<Option<UserConProfileType>, Error> {
    ConventionUsersExtensions::my_profile(self, ctx).await
  }

  /// Returns all signups for the current user within this convention. If no user is signed in,
  /// returns an empty array.
  #[graphql(name = "my_signups")]
  pub async fn my_signups(&self, ctx: &Context<'_>) -> Result<Vec<SignupType>, Error> {
    ConventionSignupsExtensions::my_signups(self, ctx).await
  }

  #[graphql(name = "orders_paginated")]
  pub async fn orders_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    #[graphql(name = "per_page")] per_page: Option<u64>,
    filters: Option<OrderFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<ModelPaginator<OrderType>, Error> {
    ConventionStoreExtensions::orders_paginated(self, ctx, page, per_page, filters, sort).await
  }

  /// Finds a product by ID in this convention. If there is no product with that ID in this
  /// convention, errors out.
  async fn product(&self, ctx: &Context<'_>, id: ID) -> Result<ProductType> {
    ConventionStoreExtensions::product(self, ctx, id).await
  }

  async fn products(
    &self,
    ctx: &Context<'_>,
    #[graphql(name = "only_available")] only_available: Option<bool>,
    #[graphql(name = "only_ticket_providing")] only_ticket_providing: Option<bool>,
  ) -> Result<Vec<ProductType>> {
    ConventionStoreExtensions::products(self, ctx, only_available, only_ticket_providing).await
  }

  pub async fn organization(&self, ctx: &Context<'_>) -> Result<Option<OrganizationType>> {
    ConventionConventionsExtensions::organization(self, ctx).await
  }

  pub async fn rooms(&self, ctx: &Context<'_>) -> Result<Vec<RoomType>, Error> {
    ConventionEventsExtensions::rooms(self, ctx).await
  }

  /// Finds an active run by ID in this convention. If there is no run with that ID in this
  /// convention, or the run's event is no longer active, errors out.
  async fn run(
    &self,
    ctx: &Context<'_>,
    #[graphql(desc = "The ID of the run to find")] id: ID,
  ) -> Result<RunType> {
    ConventionEventsExtensions::run(self, ctx, id).await
  }

  async fn signup(&self, ctx: &Context<'_>, id: Option<ID>) -> Result<SignupType, Error> {
    ConventionSignupsExtensions::signup(self, ctx, id).await
  }

  #[graphql(name = "signup_requests_paginated")]
  async fn signup_requests_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    #[graphql(name = "per_page")] per_page: Option<u64>,
    filters: Option<SignupRequestFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<ModelPaginator<SignupRequestType>, Error> {
    ConventionSignupsExtensions::signup_requests_paginated(self, ctx, page, per_page, filters, sort)
      .await
  }

  #[graphql(name = "staff_position")]
  pub async fn staff_position(
    &self,
    ctx: &Context<'_>,
    id: Option<ID>,
  ) -> Result<StaffPositionType, Error> {
    ConventionUsersExtensions::staff_position(self, ctx, id).await
  }

  #[graphql(name = "staff_positions")]
  pub async fn staff_positions(&self, ctx: &Context<'_>) -> Result<Vec<StaffPositionType>, Error> {
    ConventionUsersExtensions::staff_positions(self, ctx).await
  }

  #[graphql(name = "ticket_types")]
  pub async fn ticket_types(&self, ctx: &Context<'_>) -> Result<Vec<TicketTypeType>> {
    ConventionStoreExtensions::ticket_types(self, ctx).await
  }

  #[graphql(name = "user_activity_alert")]
  pub async fn user_activity_alert(
    &self,
    ctx: &Context<'_>,
    id: ID,
  ) -> Result<UserActivityAlertType> {
    ConventionConventionsExtensions::user_activity_alert(self, ctx, id).await
  }

  #[graphql(name = "user_activity_alerts")]
  async fn user_activity_alerts(&self, ctx: &Context<'_>) -> Result<Vec<UserActivityAlertType>> {
    ConventionConventionsExtensions::user_activity_alerts(self, ctx).await
  }

  /// Finds a UserConProfile by ID in the convention associated with this convention. If there is
  /// no UserConProfile with that ID in this convention, errors out.
  #[graphql(name = "user_con_profile")]
  pub async fn user_con_profile(
    &self,
    ctx: &Context<'_>,
    #[graphql(desc = "The ID of the UserConProfile to find.")] id: ID,
  ) -> Result<UserConProfileType, Error> {
    ConventionUsersExtensions::user_con_profile(self, ctx, id).await
  }

  /// Finds a UserConProfile by user ID in the convention associated with this convention. If
  /// there is no UserConProfile with that user ID in this convention, errors out.
  #[graphql(name = "user_con_profile_by_user_id")]
  pub async fn user_con_profile_by_user_id(
    &self,
    ctx: &Context<'_>,
    #[graphql(desc = "The user ID of the UserConProfile to find.")] user_id: ID,
  ) -> Result<UserConProfileType, Error> {
    ConventionUsersExtensions::user_con_profile_by_user_id(self, ctx, user_id).await
  }

  #[graphql(name = "user_con_profile_form")]
  pub async fn user_con_profile_form(&self, ctx: &Context<'_>) -> Result<FormType> {
    ConventionFormsExtensions::user_con_profile_form(self, ctx).await
  }

  #[graphql(name = "user_con_profiles_paginated")]
  pub async fn user_con_profiles_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    #[graphql(name = "per_page")] per_page: Option<u64>,
    filters: Option<UserConProfileFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<ModelPaginator<UserConProfileType>, Error> {
    ConventionUsersExtensions::user_con_profiles_paginated(self, ctx, page, per_page, filters, sort)
      .await
  }
}

merged_model_backed_type!(
  ConventionType,
  conventions::Model,
  "Convention",
  ConventionCmsFields,
  ConventionConventionsFields,
  ConventionEventsFields,
  ConventionNotifiersFields,
  ConventionGlueFields,
  ConventionSignupsFields,
  ConventionStoreFields
);
