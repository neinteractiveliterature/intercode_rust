use async_graphql::{Context, Error, MergedObject, Object};
use intercode_entities::{event_categories, events};
use intercode_events::partial_objects::EventCategoryEventsFields;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_pagination_from_query_builder::PaginationFromQueryBuilder;
use intercode_policies::policies::EventPolicy;
use intercode_query_builders::{sort_input::SortInput, EventFiltersInput};
use sea_orm::ModelTrait;

use crate::api::{
  merged_objects::FormType,
  objects::{DepartmentType, EventsPaginationType},
};

model_backed_type!(EventCategoryGlueFields, event_categories::Model);

#[Object]
impl EventCategoryGlueFields {
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
  ) -> Result<EventsPaginationType, Error> {
    let query_builder = EventCategoryEventsFields::from_type(self.clone())
      .events_paginated_query_builder(ctx, filters, sort)
      .await?;

    EventsPaginationType::authorized_from_query_builder(
      &query_builder,
      ctx,
      self.model.find_related(events::Entity),
      page,
      per_page,
      EventPolicy,
    )
  }
}

#[derive(MergedObject)]
#[graphql(name = "EventCategory")]
pub struct EventCategoryType(EventCategoryGlueFields, EventCategoryEventsFields);

impl ModelBackedType for EventCategoryType {
  type Model = event_categories::Model;

  fn new(model: Self::Model) -> Self {
    Self(
      EventCategoryGlueFields::new(model.clone()),
      EventCategoryEventsFields::new(model),
    )
  }

  fn get_model(&self) -> &Self::Model {
    self.0.get_model()
  }

  fn into_model(self) -> Self::Model {
    self.0.into_model()
  }
}
