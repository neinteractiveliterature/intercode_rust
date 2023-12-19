use async_graphql::*;
use intercode_entities::runs;
use intercode_events::partial_objects::RunEventsFields;
use intercode_graphql_core::{model_backed_type, ModelBackedType, ModelPaginator};
use intercode_query_builders::sort_input::SortInput;
use intercode_signups::{
  partial_objects::RunSignupsExtensions,
  query_builders::{SignupChangeFiltersInput, SignupFiltersInput},
};

use crate::merged_model_backed_type;

use super::{room_type::RoomType, EventType, SignupChangeType, SignupRequestType, SignupType};

model_backed_type!(RunGlueFields, runs::Model);

impl RunSignupsExtensions for RunGlueFields {}

#[Object]
impl RunGlueFields {
  pub async fn event(&self, ctx: &Context<'_>) -> Result<EventType, Error> {
    RunEventsFields::from_type(self.clone())
      .event(ctx)
      .await
      .map(EventType::from_type)
  }

  #[graphql(name = "my_signups")]
  async fn my_signups(&self, ctx: &Context<'_>) -> Result<Vec<SignupType>, Error> {
    RunSignupsExtensions::my_signups(self, ctx).await
  }

  #[graphql(name = "my_signup_requests")]
  async fn my_signup_requests(&self, ctx: &Context<'_>) -> Result<Vec<SignupRequestType>, Error> {
    RunSignupsExtensions::my_signup_requests(self, ctx).await
  }

  pub async fn rooms(&self, ctx: &Context<'_>) -> Result<Vec<RoomType>, Error> {
    RunEventsFields::from_type(self.clone())
      .rooms(ctx)
      .await
      .map(|rooms| rooms.into_iter().map(RoomType::from_type).collect())
  }

  #[graphql(name = "signup_changes_paginated")]
  async fn signup_changes_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    #[graphql(name = "per_page")] per_page: Option<u64>,
    filters: Option<SignupChangeFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<ModelPaginator<SignupChangeType>, Error> {
    RunSignupsExtensions::signup_changes_paginated(self, ctx, page, per_page, filters, sort)
  }

  #[graphql(name = "signups_paginated")]
  async fn signups_paginated(
    &self,
    page: Option<u64>,
    #[graphql(name = "per_page")] per_page: Option<u64>,
    filters: Option<SignupFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> ModelPaginator<SignupType> {
    RunSignupsExtensions::signups_paginated(self, page, per_page, filters, sort)
  }
}

merged_model_backed_type!(RunType, runs::Model, "Run", RunGlueFields, RunEventsFields);
