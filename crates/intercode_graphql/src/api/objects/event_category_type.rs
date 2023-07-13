use async_graphql::{Context, Error, Object, ID};
use intercode_entities::{event_categories, events};
use intercode_graphql_core::{
  load_one_by_model_id, loader_result_to_optional_single, loader_result_to_required_single,
  model_backed_type,
};
use intercode_inflector::inflector::string::pluralize;
use intercode_policies::{
  policies::{ConventionAction, ConventionPolicy, EventPolicy},
  AuthorizationInfo, Policy,
};
use intercode_query_builders::{sort_input::SortInput, EventFiltersInput, EventsQueryBuilder};
use sea_orm::ModelTrait;
use seawater::loaders::ExpectModel;

use crate::{api::interfaces::PaginationImplementation, QueryData};

use super::{DepartmentType, EventsPaginationType, FormType};

model_backed_type!(EventCategoryType, event_categories::Model);

#[Object(name = "EventCategory")]
impl EventCategoryType {
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

  pub async fn department(&self, ctx: &Context<'_>) -> Result<Option<DepartmentType>, Error> {
    let loader_result = load_one_by_model_id!(event_category_department, ctx, self)?;
    Ok(loader_result_to_optional_single!(
      loader_result,
      DepartmentType
    ))
  }

  #[graphql(name = "event_form")]
  pub async fn event_form(&self, ctx: &Context<'_>) -> Result<FormType, Error> {
    let loader_result = load_one_by_model_id!(event_category_event_form, ctx, self)?;
    Ok(loader_result_to_required_single!(loader_result, FormType))
  }

  #[graphql(name = "event_proposal_form")]
  async fn event_proposal_form(&self, ctx: &Context<'_>) -> Result<Option<FormType>, Error> {
    let loader_result = load_one_by_model_id!(event_category_event_proposal_form, ctx, self)?;
    Ok(loader_result_to_optional_single!(loader_result, FormType))
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
    let convention_loader_result = load_one_by_model_id!(event_category_convention, ctx, self)?;
    let convention = convention_loader_result.expect_one()?;
    let can_read_schedule = ConventionPolicy::action_permitted(
      ctx.data::<AuthorizationInfo>()?,
      &ConventionAction::Schedule,
      convention,
    )
    .await?;

    EventsPaginationType::authorized_from_query_builder(
      &EventsQueryBuilder::new(filters, sort, user_con_profile.cloned(), can_read_schedule),
      ctx,
      self.model.find_related(events::Entity),
      page,
      per_page,
      EventPolicy,
    )
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
  async fn scheduling_ui(&self) -> &str {
    &self.model.scheduling_ui
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
