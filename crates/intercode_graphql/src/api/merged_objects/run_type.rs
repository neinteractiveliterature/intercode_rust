use async_graphql::*;
use intercode_entities::runs;
use intercode_events::partial_objects::RunEventsFields;
use intercode_graphql_core::{model_backed_type, ModelBackedType, ModelPaginator};
use intercode_query_builders::sort_input::SortInput;
use intercode_signups::{partial_objects::RunSignupsFields, query_builders::SignupFiltersInput};

use crate::merged_model_backed_type;

use super::{room_type::RoomType, EventType, SignupRequestType, SignupType};

model_backed_type!(RunGlueFields, runs::Model);

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
    RunSignupsFields::from_type(self.clone())
      .my_signups(ctx)
      .await
      .map(|items| items.into_iter().map(SignupType::from_type).collect())
  }

  #[graphql(name = "my_signup_requests")]
  async fn my_signup_requests(&self, ctx: &Context<'_>) -> Result<Vec<SignupRequestType>, Error> {
    RunSignupsFields::from_type(self.clone())
      .my_signup_requests(ctx)
      .await
      .map(|items| {
        items
          .into_iter()
          .map(SignupRequestType::from_type)
          .collect()
      })
  }

  pub async fn rooms(&self, ctx: &Context<'_>) -> Result<Vec<RoomType>, Error> {
    RunEventsFields::from_type(self.clone())
      .rooms(ctx)
      .await
      .map(|rooms| rooms.into_iter().map(RoomType::from_type).collect())
  }

  #[graphql(name = "signups_paginated")]
  async fn signups_paginated(
    &self,
    page: Option<u64>,
    #[graphql(name = "per_page")] per_page: Option<u64>,
    filters: Option<SignupFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> ModelPaginator<SignupType> {
    RunSignupsFields::from_type(self.clone())
      .signups_paginated(page, per_page, filters, sort)
      .into_type()
  }
}

merged_model_backed_type!(RunType, runs::Model, "Run", RunGlueFields, RunEventsFields);
