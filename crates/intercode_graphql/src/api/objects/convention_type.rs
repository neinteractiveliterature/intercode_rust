use std::sync::Arc;

use crate::{
  api::merged_objects::{
    EventCategoryType, EventProposalType, EventType, FormType, MailingListsType, OrderType,
    SignupRequestType, SignupType, UserConProfileType,
  },
  merged_model_backed_type,
};

use super::{
  user_activity_alert_type::UserActivityAlertType, CmsContentGroupType, DepartmentType,
  StaffPositionType,
};
use async_graphql::*;
use chrono::{DateTime, Utc};
use intercode_cms::api::partial_objects::ConventionCmsFields;
use intercode_entities::{
  conventions, links::ConventionToStaffPositions, model_ext::user_con_profiles::BioEligibility,
  staff_positions, user_con_profiles,
};
use intercode_events::{
  partial_objects::ConventionEventsFields,
  query_builders::{EventFiltersInput, EventProposalFiltersInput},
};
use intercode_forms::partial_objects::ConventionFormsFields;
use intercode_graphql_core::{
  enums::{SiteMode, TicketMode, TimezoneMode},
  load_one_by_model_id, loader_result_to_many, model_backed_type,
  objects::ActiveStorageAttachmentType,
  query_data::QueryData,
  scalars::DateScalar,
  ModelBackedType, ModelPaginator,
};
use intercode_graphql_loaders::LoaderManager;
use intercode_policies::{policies::UserConProfilePolicy, AuthorizedFromQueryBuilder};
use intercode_query_builders::{
  sort_input::SortInput, UserConProfileFiltersInput, UserConProfilesQueryBuilder,
};
use intercode_signups::{
  partial_objects::ConventionSignupsFields, query_builders::SignupRequestFiltersInput,
};
use intercode_store::{
  objects::CouponType,
  partial_objects::ConventionStoreFields,
  query_builders::{CouponFiltersInput, OrderFiltersInput},
};
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter};
use seawater::loaders::{ExpectModel, ExpectModels};

model_backed_type!(ConventionGlueFields, conventions::Model);

#[Object]
impl ConventionGlueFields {
  async fn cms_content_groups(&self, ctx: &Context<'_>) -> Result<Vec<CmsContentGroupType>, Error> {
    ConventionCmsFields::from_type(self.clone())
      .cms_content_groups(ctx)
      .await
      .map(|partials| {
        partials
          .into_iter()
          .map(CmsContentGroupType::from_type)
          .collect()
      })
  }

  async fn cms_content_group(
    &self,
    ctx: &Context<'_>,
    id: ID,
  ) -> Result<CmsContentGroupType, Error> {
    ConventionCmsFields::from_type(self.clone())
      .cms_content_group(ctx, id)
      .await
      .map(CmsContentGroupType::from_type)
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
    ConventionStoreFields::from_type(self.clone())
      .coupons_paginated(ctx, page, per_page, filters, sort)
      .await
      .map(ModelPaginator::into_type)
  }

  /// Finds an active event by ID in this convention. If there is no event with that ID in this
  /// convention, or the event is no longer active, errors out.
  pub async fn event(&self, ctx: &Context<'_>, id: ID) -> Result<EventType, Error> {
    ConventionEventsFields::from_type(self.clone())
      .event(ctx, id)
      .await
      .map(EventType::from_type)
  }

  pub async fn events(
    &self,
    ctx: &Context<'_>,
    start: Option<DateScalar>,
    finish: Option<DateScalar>,
    include_dropped: Option<bool>,
    filters: Option<EventFiltersInput>,
  ) -> Result<Vec<EventType>, Error> {
    ConventionEventsFields::from_type(self.clone())
      .events(ctx, start, finish, include_dropped, filters)
      .await
      .map(|items| items.into_iter().map(EventType::from_type).collect())
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
    ConventionEventsFields::from_type(self.clone())
      .events_paginated(ctx, page, per_page, filters, sort)
      .await
      .map(ModelPaginator::into_type)
  }

  #[graphql(name = "event_categories")]
  async fn event_categories(
    &self,
    ctx: &Context<'_>,
    #[graphql(name = "current_ability_can_read_event_proposals")]
    current_ability_can_read_event_proposals: Option<bool>,
  ) -> Result<Vec<EventCategoryType>, Error> {
    ConventionEventsFields::from_type(self.clone())
      .event_categories(ctx, current_ability_can_read_event_proposals)
      .await
      .map(|items| {
        items
          .into_iter()
          .map(EventCategoryType::from_type)
          .collect()
      })
  }

  /// Finds an event proposal by ID in this convention. If there is no event proposal with that ID
  /// in this convention, errors out.
  #[graphql(name = "event_proposal")]
  async fn event_proposal(
    &self,
    ctx: &Context<'_>,
    #[graphql(desc = "The ID of the event proposal to find.")] id: ID,
  ) -> Result<EventProposalType> {
    ConventionEventsFields::from_type(self.clone())
      .event_proposal(ctx, id)
      .await
      .map(EventProposalType::from_type)
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
    ConventionEventsFields::from_type(self.clone())
      .event_proposals_paginated(ctx, page, per_page, filters, sort)
      .await
      .map(ModelPaginator::into_type)
  }

  pub async fn form(&self, ctx: &Context<'_>, id: ID) -> Result<FormType> {
    ConventionFormsFields::from_type(self.clone())
      .form(ctx, id)
      .await
      .map(FormType::from_type)
  }

  pub async fn forms(&self, ctx: &Context<'_>) -> Result<Vec<FormType>> {
    ConventionFormsFields::from_type(self.clone())
      .forms(ctx)
      .await
      .map(|items| items.into_iter().map(FormType::from_type).collect())
  }

  pub async fn orders_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    #[graphql(name = "per_page")] per_page: Option<u64>,
    filters: Option<OrderFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<ModelPaginator<OrderType>, Error> {
    ConventionStoreFields::from_type(self.clone())
      .orders_paginated(ctx, page, per_page, filters, sort)
      .await
      .map(ModelPaginator::into_type)
  }

  async fn signup(&self, ctx: &Context<'_>, id: ID) -> Result<SignupType, Error> {
    ConventionSignupsFields::from_type(self.clone())
      .signup(ctx, id)
      .await
      .map(SignupType::from_type)
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
    ConventionSignupsFields::from_type(self.clone())
      .signup_requests_paginated(ctx, page, per_page, filters, sort)
      .await
      .map(ModelPaginator::into_type)
  }

  #[graphql(name = "user_con_profile_form")]
  pub async fn user_con_profile_form(&self, ctx: &Context<'_>) -> Result<FormType> {
    ConventionFormsFields::from_type(self.clone())
      .user_con_profile_form(ctx)
      .await
      .map(FormType::from_type)
  }
}

model_backed_type!(ConventionApiFields, conventions::Model);

#[Object]
impl ConventionApiFields {
  async fn name(&self) -> &Option<String> {
    &self.model.name
  }

  #[graphql(name = "bio_eligible_user_con_profiles")]
  async fn bio_eligible_user_con_profiles(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<UserConProfileType>, Error> {
    let db = ctx.data::<QueryData>()?.db();

    let profiles: Vec<UserConProfileType> = self
      .model
      .find_related(user_con_profiles::Entity)
      .bio_eligible()
      .all(db.as_ref())
      .await?
      .iter()
      .map(|model| UserConProfileType::new(model.to_owned()))
      .collect::<Vec<UserConProfileType>>();

    Ok(profiles)
  }

  async fn canceled(&self) -> bool {
    self.model.canceled
  }

  #[graphql(name = "catch_all_staff_position")]
  async fn catch_all_staff_position(&self, ctx: &Context<'_>) -> Result<Option<StaffPositionType>> {
    Ok(
      ctx
        .data::<Arc<LoaderManager>>()?
        .convention_catch_all_staff_position()
        .load_one(self.model.id)
        .await?
        .try_one()
        .cloned()
        .map(StaffPositionType::new),
    )
  }

  #[graphql(name = "clickwrap_agreement")]
  async fn clickwrap_agreement(&self) -> Option<&str> {
    self.model.clickwrap_agreement.as_deref()
  }

  async fn departments(&self, ctx: &Context<'_>) -> Result<Vec<DepartmentType>> {
    let loader_result = load_one_by_model_id!(convention_departments, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, DepartmentType))
  }

  async fn domain(&self) -> &str {
    self.model.domain.as_str()
  }

  #[graphql(name = "email_from")]
  async fn email_from(&self) -> &str {
    self.model.email_from.as_str()
  }

  #[graphql(name = "email_mode")]
  async fn email_mode(&self) -> &str {
    self.model.email_mode.as_str()
  }

  #[graphql(name = "ends_at")]
  async fn ends_at(&self) -> Option<DateTime<Utc>> {
    self
      .model
      .ends_at
      .map(|t| DateTime::<Utc>::from_utc(t, Utc))
  }

  #[graphql(name = "event_mailing_list_domain")]
  async fn event_mailing_list_domain(&self) -> Option<&str> {
    self.model.event_mailing_list_domain.as_deref()
  }

  async fn favicon(&self, ctx: &Context<'_>) -> Result<Option<ActiveStorageAttachmentType>> {
    Ok(
      ctx
        .data::<Arc<LoaderManager>>()?
        .convention_favicon
        .load_one(self.model.id)
        .await?
        .and_then(|models| models.get(0).cloned())
        .map(ActiveStorageAttachmentType::new),
    )
  }

  async fn hidden(&self) -> bool {
    self.model.hidden
  }

  async fn language(&self) -> &str {
    self.model.language.as_str()
  }

  async fn location(&self) -> Option<&serde_json::Value> {
    self.model.location.as_ref()
  }

  #[graphql(name = "mailing_lists")]
  async fn mailing_lists(&self) -> MailingListsType {
    MailingListsType::from_type(self.clone())
  }

  #[graphql(name = "maximum_tickets")]
  async fn maximum_tickets(&self) -> Option<i32> {
    self.model.maximum_tickets
  }

  #[graphql(name = "my_profile")]
  async fn my_profile(&self, ctx: &Context<'_>) -> Result<Option<UserConProfileType>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    let convention_id = query_data.convention().map(|c| c.id);

    if convention_id == Some(self.model.id) {
      Ok(
        query_data
          .user_con_profile()
          .cloned()
          .map(UserConProfileType::new),
      )
    } else if let Some(user) = query_data.current_user() {
      user_con_profiles::Entity::find()
        .filter(
          user_con_profiles::Column::ConventionId
            .eq(self.model.id)
            .and(user_con_profiles::Column::UserId.eq(user.id)),
        )
        .one(query_data.db())
        .await
        .map(|result| result.map(UserConProfileType::new))
        .map_err(|e| async_graphql::Error::new(e.to_string()))
    } else {
      Ok(None)
    }
  }

  #[graphql(name = "open_graph_image")]
  async fn open_graph_image(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<ActiveStorageAttachmentType>> {
    Ok(
      ctx
        .data::<Arc<LoaderManager>>()?
        .convention_open_graph_image
        .load_one(self.model.id)
        .await?
        .and_then(|models| models.get(0).cloned())
        .map(ActiveStorageAttachmentType::new),
    )
  }

  #[graphql(name = "show_event_list")]
  async fn show_event_list(&self) -> &str {
    self.model.show_event_list.as_str()
  }

  #[graphql(name = "show_schedule")]
  async fn show_schedule(&self) -> &str {
    self.model.show_schedule.as_str()
  }

  #[graphql(name = "site_mode")]
  async fn site_mode(&self) -> Result<SiteMode, Error> {
    self.model.site_mode.as_str().try_into()
  }

  #[graphql(name = "staff_position")]
  async fn staff_position(&self, ctx: &Context<'_>, id: ID) -> Result<StaffPositionType, Error> {
    let db = ctx.data::<QueryData>()?.db();

    self
      .model
      .find_linked(ConventionToStaffPositions)
      .filter(staff_positions::Column::Id.eq(id.parse::<u64>()?))
      .one(db.as_ref())
      .await?
      .ok_or_else(|| {
        Error::new(format!(
          "Staff position with ID {} not found in convention",
          id.as_str()
        ))
      })
      .map(StaffPositionType::new)
  }

  #[graphql(name = "staff_positions")]
  async fn staff_positions(&self, ctx: &Context<'_>) -> Result<Vec<StaffPositionType>, Error> {
    let loaders = &ctx.data::<Arc<LoaderManager>>()?;

    Ok(
      loaders
        .convention_staff_positions()
        .load_one(self.model.id)
        .await?
        .expect_models()?
        .iter()
        .map(|model| StaffPositionType::new(model.clone()))
        .collect(),
    )
  }

  #[graphql(name = "starts_at")]
  async fn starts_at(&self) -> Option<DateTime<Utc>> {
    self
      .model
      .starts_at
      .map(|t| DateTime::<Utc>::from_utc(t, Utc))
  }

  #[graphql(name = "ticket_mode")]
  async fn ticket_mode(&self) -> Result<TicketMode, Error> {
    self.model.ticket_mode.as_str().try_into()
  }

  #[graphql(name = "ticket_name")]
  async fn ticket_name(&self) -> &str {
    self.model.ticket_name.as_str()
  }

  async fn ticket_name_plural(&self) -> String {
    intercode_inflector::inflector::Inflector::to_plural(self.model.ticket_name.as_str())
  }

  #[graphql(name = "timezone_mode")]
  async fn timezone_mode(&self) -> Result<TimezoneMode, Error> {
    self.model.timezone_mode.as_str().try_into()
  }

  #[graphql(name = "timezone_name")]
  async fn timezone_name(&self) -> Option<&str> {
    self.model.timezone_name.as_deref()
  }

  #[graphql(name = "user_activity_alerts")]
  async fn user_activity_alerts(&self, ctx: &Context<'_>) -> Result<Vec<UserActivityAlertType>> {
    let loader_result = load_one_by_model_id!(convention_user_activity_alerts, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, UserActivityAlertType))
  }

  #[graphql(name = "user_con_profile")]
  async fn user_con_profile(&self, ctx: &Context<'_>, id: ID) -> Result<UserConProfileType, Error> {
    let db = ctx.data::<QueryData>()?.db();

    self
      .model
      .find_related(user_con_profiles::Entity)
      .filter(user_con_profiles::Column::Id.eq(id.parse::<u64>()?))
      .one(db.as_ref())
      .await?
      .ok_or_else(|| {
        Error::new(format!(
          "No user con profile with ID {} in convention",
          id.as_str()
        ))
      })
      .map(UserConProfileType::new)
  }

  #[graphql(name = "user_con_profiles_paginated")]
  async fn user_con_profiles_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    #[graphql(name = "per_page")] per_page: Option<u64>,
    filters: Option<UserConProfileFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<ModelPaginator<UserConProfileType>, Error> {
    ModelPaginator::authorized_from_query_builder(
      &UserConProfilesQueryBuilder::new(filters, sort),
      ctx,
      self.model.find_related(user_con_profiles::Entity),
      page,
      per_page,
      UserConProfilePolicy,
    )
  }
}

merged_model_backed_type!(
  ConventionType,
  conventions::Model,
  "Convention",
  ConventionApiFields,
  ConventionCmsFields,
  ConventionGlueFields,
  ConventionStoreFields
);
