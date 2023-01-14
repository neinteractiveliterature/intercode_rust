use async_graphql::{Context, Error, Object};
use chrono::{Duration, NaiveDateTime};
use intercode_entities::runs;
use seawater::loaders::ExpectModels;

use crate::{api::scalars::JsonScalar, model_backed_type, QueryData};

use super::{signup_request_type::SignupRequestType, ModelBackedType, RoomType, SignupType};

model_backed_type!(RunType, runs::Model);

#[Object(name = "Run")]
impl RunType {
  async fn id(&self) -> i64 {
    self.model.id
  }

  #[graphql(name = "confirmed_signup_count")]
  async fn confirmed_signup_count(&self, ctx: &Context<'_>) -> Result<u64, Error> {
    let query_data = ctx.data::<QueryData>()?;

    Ok(
      query_data
        .loaders
        .run_signup_counts
        .load_one(self.model.id)
        .await?
        .unwrap_or_default()
        .counted_signups_by_state("confirmed"),
    )
  }

  #[graphql(name = "ends_at")]
  async fn ends_at(&self, ctx: &Context<'_>) -> Result<Option<NaiveDateTime>, Error> {
    let starts_at = self.model.starts_at;

    if let Some(starts_at) = starts_at {
      let query_data = ctx.data::<QueryData>()?;
      let length_seconds = query_data
        .loaders
        .run_event
        .load_one(self.model.id)
        .await?
        .expect_one()?
        .length_seconds;

      Ok(Some(starts_at + Duration::seconds(length_seconds.into())))
    } else {
      Ok(None)
    }
  }

  #[graphql(name = "my_signups")]
  async fn my_signups(&self, ctx: &Context<'_>) -> Result<Vec<SignupType>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    if let Some(user_con_profile) = query_data.user_con_profile.as_ref().as_ref() {
      let loader = query_data
        .loaders
        .run_user_con_profile_signups
        .get(user_con_profile.id)
        .await;

      Ok(
        loader
          .load_one(self.model.id)
          .await?
          .unwrap_or_default()
          .iter()
          .cloned()
          .map(SignupType::new)
          .collect(),
      )
    } else {
      Ok(vec![])
    }
  }

  #[graphql(name = "my_signup_requests")]
  async fn my_signup_requests(&self, ctx: &Context<'_>) -> Result<Vec<SignupRequestType>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    if let Some(user_con_profile) = query_data.user_con_profile.as_ref().as_ref() {
      let loader = query_data
        .loaders
        .run_user_con_profile_signup_requests
        .get(user_con_profile.id)
        .await;

      Ok(
        loader
          .load_one(self.model.id)
          .await?
          .unwrap_or_default()
          .iter()
          .cloned()
          .map(SignupRequestType::new)
          .collect(),
      )
    } else {
      Ok(vec![])
    }
  }

  #[graphql(name = "not_counted_signup_count")]
  async fn not_counted_signup_count(&self, ctx: &Context<'_>) -> Result<u64, Error> {
    let query_data = ctx.data::<QueryData>()?;

    let counts = query_data
      .loaders
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
  async fn room_names(&self, ctx: &Context<'_>) -> Result<Vec<Option<String>>, Error> {
    Ok(
      self
        .rooms(ctx)
        .await?
        .into_iter()
        .map(|room| room.get_model().name.clone())
        .collect(),
    )
  }

  async fn rooms(&self, ctx: &Context<'_>) -> Result<Vec<RoomType>, Error> {
    let query_data = ctx.data::<QueryData>()?;

    Ok(
      query_data
        .loaders
        .run_rooms
        .load_one(self.model.id)
        .await?
        .expect_models()?
        .iter()
        .map(|model| RoomType::new(model.clone()))
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
    let query_data = ctx.data::<QueryData>()?;

    let counts = query_data
      .loaders
      .run_signup_counts
      .load_one(self.model.id)
      .await?
      .unwrap_or_default();

    Ok(JsonScalar(serde_json::to_value(
      counts.count_by_state_and_bucket_key_and_counted,
    )?))
  }

  #[graphql(name = "starts_at")]
  async fn starts_at(&self) -> Option<NaiveDateTime> {
    self.model.starts_at
  }

  #[graphql(name = "title_suffix")]
  async fn title_suffix(&self) -> Option<&str> {
    self.model.title_suffix.as_deref()
  }
}
