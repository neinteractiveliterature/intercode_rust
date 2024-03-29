use async_graphql::{Context, Error, Object};
use intercode_entities::event_categories;
use intercode_events::{
  partial_objects::EventCategoryEventsFields, query_builders::EventFiltersInput,
};
use intercode_graphql_core::{model_backed_type, ModelBackedType, ModelPaginator};
use intercode_query_builders::sort_input::SortInput;

use crate::{api::merged_objects::FormType, merged_model_backed_type};

use super::{ConventionType, DepartmentType, EventType};

model_backed_type!(EventCategoryGlueFields, event_categories::Model);

#[Object]
impl EventCategoryGlueFields {
  pub async fn convention(&self, ctx: &Context<'_>) -> Result<ConventionType, Error> {
    EventCategoryEventsFields::from_type(self.clone())
      .convention(ctx)
      .await
      .map(ConventionType::new)
  }

  pub async fn department(&self, ctx: &Context<'_>) -> Result<Option<DepartmentType>, Error> {
    EventCategoryEventsFields::from_type(self.clone())
      .department(ctx)
      .await
      .map(|opt| opt.map(DepartmentType::new))
  }

  #[graphql(name = "event_form")]
  pub async fn event_form(&self, ctx: &Context<'_>) -> Result<FormType, Error> {
    EventCategoryEventsFields::from_type(self.clone())
      .event_form(ctx)
      .await
      .map(FormType::new)
  }

  #[graphql(name = "event_proposal_form")]
  async fn event_proposal_form(&self, ctx: &Context<'_>) -> Result<Option<FormType>, Error> {
    EventCategoryEventsFields::from_type(self.clone())
      .event_proposal_form(ctx)
      .await
      .map(|opt| opt.map(FormType::new))
  }

  #[graphql(name = "events_paginated")]
  async fn events_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    #[graphql(name = "per_page")] per_page: Option<u64>,
    filters: Option<EventFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<ModelPaginator<EventType>, Error> {
    EventCategoryEventsFields::from_type(self.clone())
      .events_paginated(ctx, page, per_page, filters, sort)
      .await
      .map(ModelPaginator::into_type)
  }

  async fn proposable(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    let proposal_form = self.event_proposal_form(ctx).await?;
    Ok(proposal_form.is_some())
  }
}

merged_model_backed_type!(
  EventCategoryType,
  event_categories::Model,
  "EventCategory",
  EventCategoryGlueFields,
  EventCategoryEventsFields
);
