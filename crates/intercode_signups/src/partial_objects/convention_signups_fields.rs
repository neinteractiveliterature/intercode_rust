use async_graphql::*;
use axum::async_trait;
use chrono::Utc;
use intercode_entities::{
  conventions,
  links::{ConventionToSignupRequests, ConventionToSignups},
  signup_requests, signups, MaximumEventSignupsValue,
};
use intercode_graphql_core::{
  enums::SignupMode, lax_id::LaxId, model_backed_type, objects::ScheduledStringableValueType,
  query_data::QueryData, ModelBackedType, ModelPaginator,
};
use intercode_policies::AuthorizedFromQueryBuilder;
use intercode_query_builders::sort_input::SortInput;
use intercode_timespan::ScheduledValue;
use sea_orm::{ColumnTrait, ModelTrait, QueryFilter};

use crate::{
  policies::SignupRequestPolicy,
  query_builders::{SignupRequestFiltersInput, SignupRequestsQueryBuilder},
};

model_backed_type!(ConventionSignupsFields, conventions::Model);

#[async_trait]
pub trait ConventionSignupsExtensions
where
  Self: ModelBackedType<Model = conventions::Model>,
{
  async fn my_signups<T: ModelBackedType<Model = signups::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<T>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    let Some(user_con_profile) = query_data.user_con_profile() else {
      return Ok(vec![]);
    };

    Ok(
      self
        .get_model()
        .find_linked(ConventionToSignups)
        .filter(signups::Column::UserConProfileId.eq(user_con_profile.id))
        .all(query_data.db())
        .await?
        .into_iter()
        .map(T::new)
        .collect(),
    )
  }

  async fn signup<T: ModelBackedType<Model = signups::Model>>(
    &self,
    ctx: &Context<'_>,
    id: Option<ID>,
  ) -> Result<T, Error> {
    let query_data = ctx.data::<QueryData>()?;

    Ok(T::new(
      self
        .get_model()
        .find_linked(ConventionToSignups)
        .filter(signups::Column::Id.eq(LaxId::parse(id.unwrap_or_default())?))
        .one(query_data.db())
        .await?
        .ok_or_else(|| Error::new("Signup not found"))?,
    ))
  }

  async fn signup_requests_paginated<T: ModelBackedType<Model = signup_requests::Model>>(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    per_page: Option<u64>,
    filters: Option<SignupRequestFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<ModelPaginator<T>, Error> {
    ModelPaginator::authorized_from_query_builder(
      &SignupRequestsQueryBuilder::new(filters, sort),
      ctx,
      self.get_model().find_linked(ConventionToSignupRequests),
      page,
      per_page,
      SignupRequestPolicy,
    )
  }
}

#[Object]
impl ConventionSignupsFields {
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

  #[graphql(name = "signup_mode")]
  async fn signup_mode(&self) -> Result<SignupMode, Error> {
    self
      .model
      .signup_mode
      .as_str()
      .try_into()
      .map_err(Error::from)
  }

  #[graphql(name = "signup_requests_open")]
  async fn signup_requests_open(&self) -> bool {
    self.model.signup_requests_open
  }
}
