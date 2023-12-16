use std::sync::Arc;

use async_graphql::{Context, Error, Object, Result, ID};
use chrono::Duration;
use intercode_entities::runs;
use intercode_graphql_core::{
  load_one_by_model_id, loader_result_to_many, loader_result_to_required_single, model_backed_type,
  scalars::{DateScalar, JsonScalar},
  ModelBackedType,
};
use intercode_graphql_loaders::LoaderManager;
use intercode_policies::{AuthorizationInfo, Policy};
use seawater::loaders::ExpectModel;

use crate::policies::{RunAction, RunPolicy};

use super::{EventEventsFields, RoomEventsFields};

model_backed_type!(RunEventsFields, runs::Model);

impl RunEventsFields {
  pub async fn event(&self, ctx: &Context<'_>) -> Result<EventEventsFields, Error> {
    let loader_result = load_one_by_model_id!(run_event, ctx, self)?;
    Ok(loader_result_to_required_single!(
      loader_result,
      EventEventsFields
    ))
  }

  pub async fn rooms(&self, ctx: &Context<'_>) -> Result<Vec<RoomEventsFields>, Error> {
    let loader_result = load_one_by_model_id!(run_rooms, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, RoomEventsFields))
  }
}

#[Object]
impl RunEventsFields {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "confirmed_signup_count")]
  async fn confirmed_signup_count(&self, ctx: &Context<'_>) -> Result<i64, Error> {
    Ok(
      ctx
        .data::<Arc<LoaderManager>>()?
        .run_signup_counts
        .load_one(self.model.id)
        .await?
        .unwrap_or_default()
        .counted_signups_by_state("confirmed"),
    )
  }

  #[graphql(name = "current_ability_can_signup_summary_run")]
  async fn current_ability_can_signup_summary_run(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let loaders = ctx.data::<Arc<LoaderManager>>()?;
    let event = loaders
      .run_event()
      .load_one(self.model.id)
      .await?
      .expect_one()?
      .clone();
    let convention = loaders
      .event_convention()
      .load_one(event.id)
      .await?
      .expect_one()?
      .clone();

    RunPolicy::action_permitted(
      authorization_info,
      &RunAction::SignupSummary,
      &(convention, event, self.get_model().clone()),
    )
    .await
    .map_err(|err| err.into())
  }

  #[graphql(name = "ends_at")]
  async fn ends_at(&self, ctx: &Context<'_>) -> Result<DateScalar, Error> {
    let starts_at = self.model.starts_at;

    let length_seconds = ctx
      .data::<Arc<LoaderManager>>()?
      .run_event()
      .load_one(self.model.id)
      .await?
      .expect_one()?
      .length_seconds;

    (starts_at + Duration::seconds(length_seconds.into())).try_into()
  }

  #[graphql(name = "not_counted_signup_count")]
  async fn not_counted_signup_count(&self, ctx: &Context<'_>) -> Result<i64, Error> {
    let loaders = ctx.data::<Arc<LoaderManager>>()?;

    let counts = loaders
      .run_signup_counts
      .load_one(self.model.id)
      .await?
      .unwrap_or_default();

    Ok(
      counts.not_counted_signups_by_state("confirmed")
        + counts.not_counted_signups_by_state("waitlisted"),
    )
  }

  #[graphql(name = "room_names")]
  async fn room_names(&self, ctx: &Context<'_>) -> Result<Vec<String>, Error> {
    Ok(
      self
        .rooms(ctx)
        .await?
        .into_iter()
        .map(|room| room.get_model().name.clone().unwrap_or_default())
        .collect(),
    )
  }

  #[graphql(name = "schedule_note")]
  async fn schedule_note(&self) -> Option<&str> {
    self.model.schedule_note.as_deref()
  }

  #[graphql(name = "signup_count_by_state_and_bucket_key_and_counted")]
  async fn signup_count_by_state_and_bucket_key_and_counted(
    &self,
    ctx: &Context<'_>,
  ) -> Result<JsonScalar, Error> {
    let loaders = ctx.data::<Arc<LoaderManager>>()?;

    let counts = loaders
      .run_signup_counts
      .load_one(self.model.id)
      .await?
      .unwrap_or_default();

    Ok(JsonScalar(serde_json::to_value(
      counts.count_by_state_and_bucket_key_and_counted,
    )?))
  }

  #[graphql(name = "starts_at")]
  async fn starts_at(&self) -> Result<DateScalar> {
    DateScalar::try_from(self.model.starts_at)
  }

  #[graphql(name = "title_suffix")]
  async fn title_suffix(&self) -> Option<&str> {
    self.model.title_suffix.as_deref()
  }
}
