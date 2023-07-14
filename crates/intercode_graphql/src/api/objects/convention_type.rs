use std::{str::FromStr, sync::Arc};

use super::{
  mailing_lists_type::MailingListsType, stripe_account_type::StripeAccountType,
  user_activity_alert_type::UserActivityAlertType, DepartmentType, EventCategoryType,
  EventProposalType, EventProposalsPaginationType, EventType, EventsPaginationType, FormType,
  OrdersPaginationType, RoomType, SignupRequestsPaginationType, SignupType, StaffPositionType,
  UserConProfileType, UserConProfilesPaginationType,
};
use crate::SchemaData;
use async_graphql::*;
use chrono::{DateTime, Utc};
use futures::future::try_join_all;
use intercode_cms::api::{objects::NotificationTemplateType, partial_objects::ConventionCmsFields};
use intercode_entities::{
  conventions, event_proposals, events, forms,
  links::{
    ConventionToOrders, ConventionToSignupRequests, ConventionToSignups, ConventionToStaffPositions,
  },
  model_ext::{time_bounds::TimeBoundsSelectExt, user_con_profiles::BioEligibility},
  signups, staff_positions, user_con_profiles, MaximumEventSignupsValue,
};
use intercode_graphql_core::{
  enums::{SignupMode, SiteMode, TicketMode, TimezoneMode},
  lax_id::LaxId,
  load_one_by_model_id, loader_result_to_many, model_backed_type,
  objects::{ActiveStorageAttachmentType, ScheduledStringableValueType},
  query_data::QueryData,
  scalars::DateScalar,
  ModelBackedType,
};
use intercode_graphql_loaders::LoaderManager;
use intercode_pagination_from_query_builder::PaginationFromQueryBuilder;
use intercode_policies::{
  policies::{
    ConventionAction, ConventionPolicy, EventPolicy, EventProposalAction, EventProposalPolicy,
    OrderPolicy, SignupRequestPolicy, UserConProfilePolicy,
  },
  AuthorizationInfo, Policy,
};
use intercode_query_builders::{
  sort_input::SortInput, EventFiltersInput, EventProposalFiltersInput, EventProposalsQueryBuilder,
  EventsQueryBuilder, OrderFiltersInput, OrdersQueryBuilder, QueryBuilder,
  SignupRequestFiltersInput, SignupRequestsQueryBuilder, UserConProfileFiltersInput,
  UserConProfilesQueryBuilder,
};
use intercode_store::partial_objects::ConventionStoreFields;
use intercode_timespan::ScheduledValue;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, ModelTrait, QueryFilter};
use seawater::loaders::{ExpectModel, ExpectModels};

model_backed_type!(ConventionApiFields, conventions::Model);

#[Object]
impl ConventionApiFields {
  async fn name(&self) -> &Option<String> {
    &self.model.name
  }

  #[graphql(name = "accepting_proposals")]
  async fn accepting_proposals(&self) -> bool {
    self.model.accepting_proposals.unwrap_or(false)
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

  #[graphql(name = "event_categories")]
  async fn event_categories(
    &self,
    ctx: &Context<'_>,
    #[graphql(name = "current_ability_can_read_event_proposals")]
    current_ability_can_read_event_proposals: Option<bool>,
  ) -> Result<Vec<EventCategoryType>, Error> {
    let loader_result = load_one_by_model_id!(convention_event_categories, ctx, self)?;
    let event_categories: Vec<_> = loader_result_to_many!(loader_result, EventCategoryType);

    match current_ability_can_read_event_proposals {
      Some(true) => {
        let principal = ctx.data::<AuthorizationInfo>()?;
        let futures = event_categories
          .into_iter()
          .map(|graphql_object| async {
            let event_category = graphql_object.get_model();
            let event_proposal = event_proposals::Model {
              convention_id: Some(self.model.id),
              event_category_id: event_category.id,
              ..Default::default()
            };
            Ok::<_, DbErr>((
              graphql_object,
              EventProposalPolicy::action_permitted(
                principal,
                &EventProposalAction::Read,
                &(self.model.clone(), event_proposal),
              )
              .await?,
            ))
          })
          .collect::<Vec<_>>();

        let results = try_join_all(futures.into_iter()).await?;
        let event_categories_with_permission = results
          .into_iter()
          .filter_map(
            |(graphql_object, can_read)| {
              if can_read {
                Some(graphql_object)
              } else {
                None
              }
            },
          )
          .collect::<Vec<_>>();
        Ok(event_categories_with_permission)
      }
      _ => Ok(event_categories),
    }
  }

  #[graphql(name = "event_mailing_list_domain")]
  async fn event_mailing_list_domain(&self) -> Option<&str> {
    self.model.event_mailing_list_domain.as_deref()
  }

  /// Finds an active event by ID in this convention. If there is no event with that ID in this
  /// convention, or the event is no longer active, errors out.
  async fn event(&self, ctx: &Context<'_>, id: ID) -> Result<EventType, Error> {
    let query_data = ctx.data::<QueryData>()?;
    let event_id: i64 = LaxId::parse(id)?;

    Ok(EventType::new(
      self
        .model
        .find_related(events::Entity)
        .filter(events::Column::Status.eq("active"))
        .filter(events::Column::Id.eq(event_id))
        .one(query_data.db())
        .await?
        .ok_or_else(|| {
          Error::new(format!(
            "Could not find active event with ID {} in convention",
            event_id
          ))
        })?,
    ))
  }

  async fn events(
    &self,
    ctx: &Context<'_>,
    start: Option<DateScalar>,
    finish: Option<DateScalar>,
    include_dropped: Option<bool>,
    filters: Option<EventFiltersInput>,
  ) -> Result<Vec<EventType>, Error> {
    let mut scope = self
      .model
      .find_related(events::Entity)
      .between(start.map(Into::into), finish.map(Into::into));

    if include_dropped != Some(true) {
      scope = scope.filter(events::Column::Status.eq("active"));
    }

    let query_builder = EventsQueryBuilder::new(
      filters,
      None,
      ctx.data::<QueryData>()?.user_con_profile().cloned(),
      ConventionPolicy::action_permitted(
        ctx.data::<AuthorizationInfo>()?,
        &ConventionAction::Schedule,
        &self.model,
      )
      .await?,
    );

    scope = query_builder.apply_filters(scope);

    Ok(
      scope
        .all(ctx.data::<QueryData>()?.db())
        .await?
        .into_iter()
        .map(EventType::new)
        .collect(),
    )
  }

  #[graphql(name = "events_paginated")]
  async fn events_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    #[graphql(name = "per_page")] per_page: Option<u64>,
    filters: Option<EventFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<EventsPaginationType, Error> {
    let user_con_profile = ctx.data::<QueryData>()?.user_con_profile();
    let can_read_schedule = ConventionPolicy::action_permitted(
      ctx.data::<AuthorizationInfo>()?,
      &ConventionAction::Schedule,
      &self.model,
    )
    .await?;

    EventsPaginationType::authorized_from_query_builder(
      &EventsQueryBuilder::new(filters, sort, user_con_profile.cloned(), can_read_schedule),
      ctx,
      self
        .model
        .find_related(events::Entity)
        .filter(events::Column::Status.eq("active")),
      page,
      per_page,
      EventPolicy,
    )
  }

  /// Finds an event proposal by ID in this convention. If there is no event proposal with that ID
  /// in this convention, errors out.
  #[graphql(name = "event_proposal")]
  async fn event_proposal(
    &self,
    ctx: &Context<'_>,
    #[graphql(desc = "The ID of the event proposal to find.")] id: ID,
  ) -> Result<EventProposalType> {
    let db = ctx.data::<QueryData>()?.db();
    let id = LaxId::parse(id)?;
    let event_proposal = event_proposals::Entity::find()
      .filter(event_proposals::Column::ConventionId.eq(self.model.id))
      .filter(event_proposals::Column::Id.eq(id))
      .one(db)
      .await?
      .ok_or_else(|| Error::new(format!("Event proposal {} not found", id)))?;
    Ok(EventProposalType::new(event_proposal))
  }

  #[graphql(name = "event_proposals_paginated")]
  async fn event_proposals_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    #[graphql(name = "per_page")] per_page: Option<u64>,
    filters: Option<EventProposalFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<EventProposalsPaginationType, Error> {
    EventProposalsPaginationType::authorized_from_query_builder(
      &EventProposalsQueryBuilder::new(filters, sort),
      ctx,
      self.model.find_related(event_proposals::Entity),
      page,
      per_page,
      EventProposalPolicy,
    )
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

  async fn form(&self, ctx: &Context<'_>, id: ID) -> Result<FormType> {
    let form = self
      .model
      .all_forms()
      .filter(forms::Column::Id.eq(LaxId::parse(id.clone())?))
      .one(ctx.data::<QueryData>()?.db())
      .await?;
    form
      .ok_or_else(|| Error::new(format!("Form {:?} not found in convention", id)))
      .map(FormType::new)
  }

  async fn forms(&self, ctx: &Context<'_>) -> Result<Vec<FormType>> {
    let forms = self
      .model
      .all_forms()
      .all(ctx.data::<QueryData>()?.db())
      .await?;
    Ok(forms.into_iter().map(FormType::new).collect())
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
    MailingListsType::new(self.model.clone())
  }

  #[graphql(name = "maximum_event_signups")]
  async fn maximum_event_signups(
    &self,
  ) -> Result<Option<ScheduledStringableValueType<Utc, MaximumEventSignupsValue>>> {
    let scheduled_value: Option<ScheduledValue<Utc, MaximumEventSignupsValue>> = self
      .model
      .maximum_event_signups
      .clone()
      .map(serde_json::from_value)
      .transpose()?;

    Ok(scheduled_value.map(ScheduledStringableValueType::new))
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

  #[graphql(name = "notification_templates")]
  async fn notification_templates(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<NotificationTemplateType>> {
    let loader_result = load_one_by_model_id!(convention_notification_templates, ctx, self)?;
    Ok(loader_result_to_many!(
      loader_result,
      NotificationTemplateType
    ))
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

  #[graphql(name = "orders_paginated")]
  async fn orders_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    #[graphql(name = "per_page")] per_page: Option<u64>,
    filters: Option<OrderFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<OrdersPaginationType, Error> {
    OrdersPaginationType::authorized_from_query_builder(
      &OrdersQueryBuilder::new(filters, sort),
      ctx,
      self.model.find_linked(ConventionToOrders),
      page,
      per_page,
      OrderPolicy,
    )
  }

  async fn rooms(&self, ctx: &Context<'_>) -> Result<Vec<RoomType>, Error> {
    let loader_result = load_one_by_model_id!(convention_rooms, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, RoomType))
  }

  async fn signup(&self, ctx: &Context<'_>, id: ID) -> Result<SignupType, Error> {
    let query_data = ctx.data::<QueryData>()?;

    Ok(SignupType::new(
      self
        .model
        .find_linked(ConventionToSignups)
        .filter(signups::Column::Id.eq(id.parse::<i64>()?))
        .one(query_data.db())
        .await?
        .ok_or_else(|| Error::new("Signup not found"))?,
    ))
  }

  #[graphql(name = "signup_mode")]
  async fn signup_mode(&self) -> Result<SignupMode, Error> {
    self.model.signup_mode.as_str().try_into()
  }

  #[graphql(name = "signup_requests_open")]
  async fn signup_requests_open(&self) -> bool {
    self.model.signup_requests_open
  }

  #[graphql(name = "show_event_list")]
  async fn show_event_list(&self) -> &str {
    self.model.show_event_list.as_str()
  }

  #[graphql(name = "show_schedule")]
  async fn show_schedule(&self) -> &str {
    self.model.show_schedule.as_str()
  }

  #[graphql(name = "signup_requests_paginated")]
  async fn signup_requests_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    #[graphql(name = "per_page")] per_page: Option<u64>,
    filters: Option<SignupRequestFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<SignupRequestsPaginationType, Error> {
    SignupRequestsPaginationType::authorized_from_query_builder(
      &SignupRequestsQueryBuilder::new(filters, sort),
      ctx,
      self.model.find_linked(ConventionToSignupRequests),
      page,
      per_page,
      SignupRequestPolicy,
    )
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

  #[graphql(name = "stripe_account")]
  async fn stripe_account(&self, ctx: &Context<'_>) -> Result<Option<StripeAccountType>> {
    if let Some(id) = &self.model.stripe_account_id {
      let client = &ctx.data::<SchemaData>()?.stripe_client;
      let acct = stripe::Account::retrieve(client, &stripe::AccountId::from_str(id)?, &[]).await?;
      Ok(Some(StripeAccountType::new(acct)))
    } else {
      Ok(None)
    }
  }

  #[graphql(name = "stripe_account_id")]
  async fn stripe_account_id(&self) -> Option<&str> {
    self.model.stripe_account_id.as_deref()
  }

  #[graphql(name = "stripe_account_ready_to_charge")]
  async fn stripe_account_ready_to_charge(&self) -> bool {
    self.model.stripe_account_ready_to_charge
  }

  #[graphql(name = "stripe_publishable_key")]
  async fn stripe_publishable_key(&self) -> Option<String> {
    std::env::var("STRIPE_PUBLISHABLE_KEY").ok()
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

  #[graphql(name = "user_con_profile_form")]
  async fn user_con_profile_form(&self, ctx: &Context<'_>) -> Result<FormType> {
    Ok(FormType::new(
      ctx
        .data::<Arc<LoaderManager>>()?
        .convention_user_con_profile_form()
        .load_one(self.model.id)
        .await?
        .expect_one()?
        .clone(),
    ))
  }

  #[graphql(name = "user_con_profiles_paginated")]
  async fn user_con_profiles_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    #[graphql(name = "per_page")] per_page: Option<u64>,
    filters: Option<UserConProfileFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<UserConProfilesPaginationType, Error> {
    UserConProfilesPaginationType::authorized_from_query_builder(
      &UserConProfilesQueryBuilder::new(filters, sort),
      ctx,
      self.model.find_related(user_con_profiles::Entity),
      page,
      per_page,
      UserConProfilePolicy,
    )
  }
}

#[derive(MergedObject)]
#[graphql(name = "Convention")]
pub struct ConventionType(
  ConventionApiFields,
  ConventionCmsFields,
  ConventionStoreFields,
);

impl ModelBackedType for ConventionType {
  type Model = conventions::Model;

  fn new(model: Self::Model) -> Self {
    Self(
      ConventionApiFields::new(model.clone()),
      ConventionCmsFields::new(model.clone()),
      ConventionStoreFields::new(model),
    )
  }

  fn get_model(&self) -> &Self::Model {
    self.0.get_model()
  }
}
