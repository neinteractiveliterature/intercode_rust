use std::str::FromStr;

use super::{
  active_storage_attachment_type::ActiveStorageAttachmentType,
  stripe_account_type::StripeAccountType, CmsContentGroupType, CmsContentType, CmsFileType,
  CmsGraphqlQueryType, CmsLayoutType, CmsNavigationItemType, CmsPartialType, CmsVariableType,
  CouponsPaginationType, EventCategoryType, EventProposalType, EventProposalsPaginationType,
  EventType, EventsPaginationType, FormType, ModelBackedType, OrdersPaginationType, PageType,
  RoomType, ScheduledStringableValueType, SignupRequestsPaginationType, SignupType,
  StaffPositionType, TicketTypeType, UserConProfileType, UserConProfilesPaginationType,
};
use crate::{
  api::{
    enums::{SignupMode, SiteMode, TicketMode, TimezoneMode},
    inputs::{
      CouponFiltersInput, EventFiltersInput, EventProposalFiltersInput, OrderFiltersInput,
      SignupRequestFiltersInput, SortInput, UserConProfileFiltersInput,
    },
    interfaces::{CmsParentImplementation, PaginationImplementation},
    objects::ProductType,
    scalars::DateScalar,
  },
  cms_rendering_context::CmsRenderingContext,
  lax_id::LaxId,
  load_one_by_model_id, loader_result_to_many,
  query_builders::{
    CouponsQueryBuilder, EventProposalsQueryBuilder, OrdersQueryBuilder, QueryBuilder,
    SignupRequestsQueryBuilder,
  },
  LiquidRenderer, QueryData, SchemaData,
};
use async_graphql::*;
use chrono::{DateTime, Utc};
use futures::future::try_join_all;
use intercode_entities::{
  cms_parent::CmsParentTrait,
  cms_partials, conventions, coupons, event_proposals, events,
  links::{
    ConventionToOrders, ConventionToSignupRequests, ConventionToSignups, ConventionToStaffPositions,
  },
  model_ext::time_bounds::TimeBoundsSelectExt,
  orders, runs, signup_requests, signups, staff_positions, team_members, user_con_profiles, users,
  MaximumEventSignupsValue,
};
use intercode_policies::{
  policies::{
    CouponPolicy, EventProposalAction, EventProposalPolicy, OrderAction, OrderPolicy,
    SignupRequestAction, SignupRequestPolicy,
  },
  AuthorizationInfo, EntityPolicy, Policy, ReadManageAction,
};
use intercode_timespan::ScheduledValue;
use liquid::object;
use sea_orm::{
  sea_query::Expr, ColumnTrait, DbErr, EntityTrait, JoinType, ModelTrait, QueryFilter, QueryOrder,
  QuerySelect, RelationTrait,
};
use seawater::loaders::{ExpectModel, ExpectModels};

use crate::model_backed_type;
model_backed_type!(ConventionType, conventions::Model);

#[Object(name = "Convention")]
impl ConventionType {
  pub async fn id(&self) -> ID {
    ID(self.model.id.to_string())
  }

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
      .left_join(staff_positions::Entity)
      .left_join(team_members::Entity)
      .filter(
        staff_positions::Column::Id
          .is_not_null()
          .or(team_members::Column::Id.is_not_null()),
      )
      .group_by(user_con_profiles::Column::Id)
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
        .data::<QueryData>()?
        .loaders()
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

  #[graphql(name = "coupons_paginated")]
  async fn coupons_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    #[graphql(name = "per_page")] per_page: Option<u64>,
    filters: Option<CouponFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<CouponsPaginationType, Error> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let scope = self.model.find_related(coupons::Entity).filter(
      coupons::Column::Id.in_subquery(
        sea_orm::QuerySelect::query(
          &mut CouponPolicy::accessible_to(authorization_info, &ReadManageAction::Read)
            .select_only()
            .column(coupons::Column::Id),
        )
        .take(),
      ),
    );
    CouponsQueryBuilder::new(filters, sort).paginate(ctx, scope, page, per_page)
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

    if let Some(filters) = filters {
      scope = filters.apply_filters(ctx, &scope)?;
    }

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
    let mut scope = self
      .model
      .find_related(events::Entity)
      .filter(events::Column::Status.eq("active"));

    if let Some(filters) = filters {
      scope = filters.apply_filters(ctx, &scope)?;
    }

    if let Some(sort) = sort {
      for sort_column in sort {
        let order = sort_column.query_order();

        scope = match sort_column.field.as_str() {
          "first_scheduled_run_start" => {
            // TODO authorize that the user is able to see the schedule
            scope
              .left_join(runs::Entity)
              .filter(Expr::cust(
                "runs.starts_at = (
                SELECT MIN(runs.starts_at) FROM runs WHERE runs.event_id = events.id
              )",
              ))
              .order_by(runs::Column::StartsAt, order)
          }
          "created_at" => scope.order_by(events::Column::CreatedAt, order),
          "owner" => scope
            .join(JoinType::LeftJoin, events::Relation::Users1.def())
            .order_by(users::Column::LastName, order.clone())
            .order_by(users::Column::FirstName, order),
          "title" => scope.order_by(
            Expr::cust(
              "regexp_replace(
                  regexp_replace(
                    trim(regexp_replace(unaccent(events.title), '[^0-9a-z ]', '', 'gi')),
                    '^(the|a|an) +',
                    '',
                    'i'
                  ),
                  ' ',
                  '',
                  'g'
                )",
            ),
            order,
          ),
          _ => scope,
        }
      }
    }

    Ok(EventsPaginationType::new(Some(scope), page, per_page))
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
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let scope = self.model.find_related(event_proposals::Entity).filter(
      event_proposals::Column::Id.in_subquery(
        sea_orm::QuerySelect::query(
          &mut EventProposalPolicy::accessible_to(authorization_info, &EventProposalAction::Read)
            .select_only()
            .column(event_proposals::Column::Id),
        )
        .take(),
      ),
    );
    EventProposalsQueryBuilder::new(filters, sort).paginate(ctx, scope, page, per_page)
  }

  async fn favicon(&self, ctx: &Context<'_>) -> Result<Option<ActiveStorageAttachmentType>> {
    Ok(
      ctx
        .data::<QueryData>()?
        .loaders()
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

  #[graphql(name = "open_graph_image")]
  async fn open_graph_image(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<ActiveStorageAttachmentType>> {
    Ok(
      ctx
        .data::<QueryData>()?
        .loaders()
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
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let scope = self.model.find_linked(ConventionToOrders).filter(
      orders::Column::Id.in_subquery(
        sea_orm::QuerySelect::query(
          &mut OrderPolicy::accessible_to(authorization_info, &OrderAction::Read)
            .select_only()
            .column(orders::Column::Id),
        )
        .take(),
      ),
    );
    OrdersQueryBuilder::new(filters, sort).paginate(ctx, scope, page, per_page)
  }

  #[graphql(name = "pre_schedule_content_html")]
  async fn pre_schedule_content_html(&self, ctx: &Context<'_>) -> Result<Option<String>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    let liquid_renderer = ctx.data::<Box<dyn LiquidRenderer>>()?;

    let partial = self
      .model
      .cms_partials()
      .filter(cms_partials::Column::Name.eq("pre_schedule_text"))
      .one(query_data.db())
      .await?;

    if let Some(partial) = partial {
      let cms_rendering_context =
        CmsRenderingContext::new(object!({}), query_data, liquid_renderer.as_ref());

      cms_rendering_context
        .render_liquid(&partial.content.unwrap_or_default(), None)
        .await
        .map(Some)
    } else {
      Ok(None)
    }
  }

  async fn products(&self, ctx: &Context<'_>) -> Result<Vec<ProductType>> {
    let loader_result = load_one_by_model_id!(convention_products, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, ProductType))
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
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let scope = self.model.find_linked(ConventionToSignupRequests).filter(
      signup_requests::Column::Id.in_subquery(
        sea_orm::QuerySelect::query(
          &mut SignupRequestPolicy::accessible_to(authorization_info, &SignupRequestAction::Read)
            .select_only()
            .column(signup_requests::Column::Id),
        )
        .take(),
      ),
    );
    SignupRequestsQueryBuilder::new(filters, sort).paginate(ctx, scope, page, per_page)
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
    let query_data = &ctx.data::<QueryData>()?;

    Ok(
      query_data
        .loaders()
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

  #[graphql(name = "ticket_types")]
  async fn ticket_types(&self, ctx: &Context<'_>) -> Result<Vec<TicketTypeType>, Error> {
    let query_data = ctx.data::<QueryData>()?;

    Ok(
      query_data
        .loaders()
        .convention_ticket_types()
        .load_one(self.model.id)
        .await?
        .expect_models()?
        .iter()
        .map(|tt| TicketTypeType::new(tt.to_owned()))
        .collect(),
    )
  }

  #[graphql(name = "tickets_available_for_purchase")]
  async fn tickets_available_for_purchase(&self) -> bool {
    self.model.tickets_available_for_purchase()
  }

  #[graphql(name = "timezone_mode")]
  async fn timezone_mode(&self) -> Result<TimezoneMode, Error> {
    self.model.timezone_mode.as_str().try_into()
  }

  #[graphql(name = "timezone_name")]
  async fn timezone_name(&self) -> Option<&str> {
    self.model.timezone_name.as_deref()
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
        .data::<QueryData>()?
        .loaders()
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
    let mut scope = self.model.find_related(user_con_profiles::Entity);

    if let Some(filters) = filters {
      scope = filters.apply_filters(ctx, &scope)?;
    }

    if let Some(sort) = sort {
      for sort_column in sort {
        let _order = sort_column.query_order();

        scope = match sort_column.field.as_str() {
          "id" => todo!(),
          "attending" => todo!(),
          "email" => todo!(),
          "first_name" => todo!(),
          "is_team_member" => todo!(),
          "last_name" => todo!(),
          "payment_amount" => todo!(),
          "privileges" => todo!(),
          "name" => todo!(),
          "ticket" => todo!(),
          "ticket_type" => todo!(),
          "user_id" => todo!(),
          _ => scope,
        }
      }
    }

    Ok(UserConProfilesPaginationType::new(
      Some(scope),
      page,
      per_page,
    ))
  }

  // STUFF FOR IMPLEMENTING CMS_PARENT

  async fn cms_content_groups(&self, ctx: &Context<'_>) -> Result<Vec<CmsContentGroupType>, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::cms_content_groups(self, ctx).await
  }

  async fn cms_content_group(
    &self,
    ctx: &Context<'_>,
    id: ID,
  ) -> Result<CmsContentGroupType, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::cms_content_group(self, ctx, id).await
  }

  async fn cms_files(&self, ctx: &Context<'_>) -> Result<Vec<CmsFileType>, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::cms_files(self, ctx).await
  }

  async fn cms_file(&self, ctx: &Context<'_>, id: ID) -> Result<CmsFileType, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::cms_file(self, ctx, id).await
  }

  async fn cms_graphql_queries(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<CmsGraphqlQueryType>, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::cms_graphql_queries(self, ctx).await
  }

  async fn cms_graphql_query(
    &self,
    ctx: &Context<'_>,
    id: ID,
  ) -> Result<CmsGraphqlQueryType, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::cms_graphql_query(self, ctx, id).await
  }

  async fn cms_layouts(&self, ctx: &Context<'_>) -> Result<Vec<CmsLayoutType>, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::cms_layouts(self, ctx).await
  }

  async fn cms_layout(&self, ctx: &Context<'_>, id: ID) -> Result<CmsLayoutType, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::cms_layout(self, ctx, id).await
  }

  async fn cms_navigation_items(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<CmsNavigationItemType>, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::cms_navigation_items(self, ctx).await
  }

  async fn cms_pages(&self, ctx: &Context<'_>) -> Result<Vec<PageType>, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::cms_pages(self, ctx).await
  }

  async fn cms_page(
    &self,
    ctx: &Context<'_>,
    id: Option<ID>,
    slug: Option<String>,
    root_page: Option<bool>,
  ) -> Result<PageType, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::cms_page(self, ctx, id, slug, root_page)
      .await
  }

  async fn cms_partials(&self, ctx: &Context<'_>) -> Result<Vec<CmsPartialType>, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::cms_partials(self, ctx).await
  }

  async fn cms_variables(&self, ctx: &Context<'_>) -> Result<Vec<CmsVariableType>, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::cms_variables(self, ctx).await
  }

  async fn default_layout(&self, ctx: &Context<'_>) -> Result<CmsLayoutType, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::default_layout(self, ctx).await
  }

  async fn effective_cms_layout(
    &self,
    ctx: &Context<'_>,
    path: String,
  ) -> Result<CmsLayoutType, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::effective_cms_layout(self, ctx, path)
      .await
  }

  async fn root_page(&self, ctx: &Context<'_>) -> Result<PageType, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::root_page(self, ctx).await
  }

  async fn typeahead_search_cms_content(
    &self,
    ctx: &Context<'_>,
    name: Option<String>,
  ) -> Result<Vec<CmsContentType>, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::typeahead_search_cms_content(
      self, ctx, name,
    )
    .await
  }
}

impl CmsParentImplementation<conventions::Model> for ConventionType {}
