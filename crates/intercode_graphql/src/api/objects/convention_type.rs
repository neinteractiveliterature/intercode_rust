use std::sync::Arc;

use super::{
  CmsContentGroupType, CmsContentType, CmsFileType, CmsGraphqlQueryType, CmsLayoutType,
  CmsNavigationItemType, CmsPartialType, CmsVariableType, EventCategoryType, EventType,
  EventsPaginationType, ModelBackedType, PageType, RoomType, StaffPositionType, TicketTypeType,
  UserConProfileType,
};
use crate::{
  api::{
    enums::{SignupMode, SiteMode, TicketMode, TimezoneMode},
    inputs::{EventFiltersInput, SortInput},
    interfaces::CmsParentImplementation,
    scalars::DateScalar,
  },
  cms_rendering_context::CmsRenderingContext,
  lax_id::LaxId,
  LiquidRenderer, QueryData,
};
use async_graphql::*;
use chrono::{DateTime, Utc};
use intercode_entities::{
  cms_parent::CmsParentTrait, cms_partials, conventions, events, links::ConventionToStaffPositions,
  model_ext::time_bounds::TimeBoundsSelectExt, runs, staff_positions, team_members,
  user_con_profiles, users,
};
use liquid::object;
use sea_orm::{
  sea_query::Expr, ColumnTrait, EntityTrait, JoinType, ModelTrait, Order, QueryFilter, QueryOrder,
  QuerySelect, RelationTrait,
};
use seawater::loaders::ExpectModels;

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
    let db = &ctx.data::<QueryData>()?.db;

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

  #[graphql(name = "clickwrap_agreement")]
  async fn clickwrap_agreement(&self) -> Option<&str> {
    self.model.clickwrap_agreement.as_deref()
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
  async fn event_categories(&self, ctx: &Context<'_>) -> Result<Vec<EventCategoryType>, Error> {
    Ok(
      ctx
        .data::<QueryData>()?
        .loaders
        .convention_event_categories
        .load_one(self.model.id)
        .await?
        .expect_models()?
        .iter()
        .cloned()
        .map(EventCategoryType::new)
        .collect(),
    )
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
        .one(&query_data.db)
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
    #[graphql(name = "include_dropped")] include_dropped: Option<bool>,
    filters: Option<EventFiltersInput>,
  ) -> Result<Vec<EventType>, Error> {
    let mut scope = self
      .model
      .find_related(events::Entity)
      .between(start.map(Into::into), finish.map(Into::into));

    if let Some(true) = include_dropped {
      scope = scope.filter(events::Column::Status.eq("active"));
    }

    if let Some(filters) = filters {
      scope = filters.apply_filters(ctx, &scope)?;
    }

    Ok(
      scope
        .all(ctx.data::<QueryData>()?.db.as_ref())
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
        let order = if sort_column.desc {
          Order::Desc
        } else {
          Order::Asc
        };

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

  async fn hidden(&self) -> bool {
    self.model.hidden
  }

  async fn language(&self) -> &str {
    self.model.language.as_str()
  }

  async fn location(&self) -> Option<&serde_json::Value> {
    self.model.location.as_ref()
  }

  #[graphql(name = "maximum_tickets")]
  async fn maximum_tickets(&self) -> Option<i32> {
    self.model.maximum_tickets
  }

  #[graphql(name = "my_profile")]
  async fn my_profile(&self, ctx: &Context<'_>) -> Result<Option<UserConProfileType>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    let convention_id = query_data.convention.as_ref().as_ref().map(|c| c.id);

    if convention_id == Some(self.model.id) {
      Ok(
        query_data
          .user_con_profile
          .as_ref()
          .as_ref()
          .map(|ucp| UserConProfileType::new(ucp.to_owned())),
      )
    } else if let Some(user) = query_data.current_user.as_ref() {
      let query_data = ctx.data::<QueryData>()?;

      user_con_profiles::Entity::find()
        .filter(
          user_con_profiles::Column::ConventionId
            .eq(self.model.id)
            .and(user_con_profiles::Column::UserId.eq(user.id)),
        )
        .one(query_data.db.as_ref())
        .await
        .map(|result| result.map(UserConProfileType::new))
        .map_err(|e| async_graphql::Error::new(e.to_string()))
    } else {
      Ok(None)
    }
  }

  #[graphql(name = "pre_schedule_content_html")]
  async fn pre_schedule_content_html(&self, ctx: &Context<'_>) -> Result<Option<String>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    let liquid_renderer = ctx.data::<Arc<dyn LiquidRenderer>>()?;

    let partial = self
      .model
      .cms_partials()
      .filter(cms_partials::Column::Name.eq("pre_schedule_text"))
      .one(&query_data.db)
      .await?;

    if let Some(partial) = partial {
      let cms_rendering_context =
        CmsRenderingContext::new(object!({}), query_data, liquid_renderer.clone());

      cms_rendering_context
        .render_liquid(&partial.content.unwrap_or_default(), None)
        .await
        .map(Some)
    } else {
      Ok(None)
    }
  }

  async fn rooms(&self, ctx: &Context<'_>) -> Result<Vec<RoomType>, Error> {
    Ok(
      ctx
        .data::<QueryData>()?
        .loaders
        .convention_rooms
        .load_one(self.model.id)
        .await?
        .expect_models()?
        .iter()
        .map(|room| RoomType::new(room.clone()))
        .collect(),
    )
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

  #[graphql(name = "site_mode")]
  async fn site_mode(&self) -> Result<SiteMode, Error> {
    self.model.site_mode.as_str().try_into()
  }

  #[graphql(name = "staff_position")]
  async fn staff_position(&self, ctx: &Context<'_>, id: ID) -> Result<StaffPositionType, Error> {
    let db = &ctx.data::<QueryData>()?.db;

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
        .loaders
        .convention_staff_positions
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

  #[graphql(name = "stripe_account_id")]
  async fn stripe_account_id(&self) -> Option<&str> {
    self.model.stripe_account_id.as_deref()
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
        .loaders
        .convention_ticket_types
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
    let db = &ctx.data::<QueryData>()?.db;

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
