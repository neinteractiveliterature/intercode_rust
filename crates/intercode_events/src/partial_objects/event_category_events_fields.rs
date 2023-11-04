use async_graphql::{Context, Error, Object, Result, ID};
use intercode_entities::{departments, event_categories, events, forms};
use intercode_graphql_core::{
  enums::SchedulingUI, load_one_by_model_id, model_backed_type, query_data::QueryData,
  ModelPaginator,
};
use intercode_inflector::inflector::string::pluralize;
use intercode_policies::{
  policies::{ConventionAction, ConventionPolicy, EventPolicy},
  AuthorizationInfo, AuthorizedFromQueryBuilder, Policy,
};
use intercode_query_builders::sort_input::SortInput;
use sea_orm::ModelTrait;
use seawater::loaders::ExpectModel;

use crate::query_builders::{EventFiltersInput, EventsQueryBuilder};

use super::EventEventsFields;

model_backed_type!(EventCategoryEventsFields, event_categories::Model);

impl EventCategoryEventsFields {
  pub async fn department(&self, ctx: &Context<'_>) -> Result<Option<departments::Model>, Error> {
    let loader_result = load_one_by_model_id!(event_category_department, ctx, self)?;
    Ok(loader_result.try_one().cloned())
  }

  pub async fn event_form(&self, ctx: &Context<'_>) -> Result<forms::Model, Error> {
    let loader_result = load_one_by_model_id!(event_category_event_form, ctx, self)?;
    Ok(loader_result.expect_one()?.clone())
  }

  pub async fn event_proposal_form(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<forms::Model>, Error> {
    let loader_result = load_one_by_model_id!(event_category_event_proposal_form, ctx, self)?;
    Ok(loader_result.try_one().cloned())
  }

  pub async fn events_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    per_page: Option<u64>,
    filters: Option<EventFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<ModelPaginator<EventEventsFields>, Error> {
    let user_con_profile = ctx.data::<QueryData>()?.user_con_profile();
    let convention_loader_result = load_one_by_model_id!(event_category_convention, ctx, self)?;
    let convention = convention_loader_result.expect_one()?;
    let can_read_schedule = ConventionPolicy::action_permitted(
      ctx.data::<AuthorizationInfo>()?,
      &ConventionAction::Schedule,
      convention,
    )
    .await?;

    ModelPaginator::authorized_from_query_builder(
      &EventsQueryBuilder::new(filters, sort, user_con_profile.cloned(), can_read_schedule),
      ctx,
      self.model.find_related(events::Entity),
      page,
      per_page,
      EventPolicy,
    )
  }
}

#[Object]
impl EventCategoryEventsFields {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "can_provide_tickets")]
  async fn can_provide_tickets(&self) -> bool {
    self.model.can_provide_tickets
  }

  #[graphql(name = "default_color")]
  async fn default_color(&self) -> &str {
    &self.model.default_color
  }

  #[graphql(name = "full_color")]
  async fn full_color(&self) -> &str {
    &self.model.full_color
  }

  async fn name(&self) -> &str {
    &self.model.name
  }

  #[graphql(name = "proposal_description")]
  async fn proposal_description(&self) -> Option<&str> {
    self.model.proposal_description.as_deref()
  }

  #[graphql(name = "scheduling_ui")]
  async fn scheduling_ui(&self) -> Result<SchedulingUI> {
    self.model.scheduling_ui.as_str().try_into()
  }

  #[graphql(name = "signed_up_color")]
  async fn signed_up_color(&self) -> &str {
    &self.model.signed_up_color
  }

  #[graphql(name = "team_member_name")]
  async fn team_member_name(&self) -> &str {
    &self.model.team_member_name
  }

  async fn team_member_name_plural(&self) -> String {
    pluralize::to_plural(&self.model.team_member_name)
  }
}
