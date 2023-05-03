use async_graphql::{Context, Error, Object, ID};
use chrono::{Duration, NaiveDateTime};
use intercode_entities::{events, runs, signups, user_con_profiles, users};
use intercode_policies::{
  policies::{RunAction, RunPolicy},
  AuthorizationInfo, Policy,
};
use sea_orm::{
  sea_query::{Expr, Func, SimpleExpr},
  JoinType, ModelTrait, QueryOrder, QuerySelect, RelationTrait,
};
use seawater::loaders::{ExpectModel, ExpectModels};

use crate::{
  api::{
    inputs::{SignupFiltersInput, SortInput},
    interfaces::PaginationImplementation,
    scalars::JsonScalar,
  },
  model_backed_type, QueryData,
};

use super::{
  signup_request_type::SignupRequestType, EventType, ModelBackedType, RoomType, SignupType,
  SignupsPaginationType,
};

model_backed_type!(RunType, runs::Model);

#[Object(name = "Run")]
impl RunType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "confirmed_signup_count")]
  async fn confirmed_signup_count(&self, ctx: &Context<'_>) -> Result<i64, Error> {
    let query_data = ctx.data::<QueryData>()?;

    Ok(
      query_data
        .loaders()
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
    let query_data = ctx.data::<QueryData>()?;
    let event = query_data
      .loaders()
      .run_event()
      .load_one(self.model.id)
      .await?
      .expect_one()?
      .clone();
    let convention = query_data
      .loaders()
      .event_convention()
      .load_one(event.id)
      .await?
      .expect_one()?
      .clone();

    RunPolicy::action_permitted(
      authorization_info,
      &RunAction::SignupSummary,
      &(convention, event, self.model.clone()),
    )
    .await
    .map_err(|err| err.into())
  }

  #[graphql(name = "ends_at")]
  async fn ends_at(&self, ctx: &Context<'_>) -> Result<Option<NaiveDateTime>, Error> {
    let starts_at = self.model.starts_at;

    if let Some(starts_at) = starts_at {
      let query_data = ctx.data::<QueryData>()?;
      let length_seconds = query_data
        .loaders()
        .run_event()
        .load_one(self.model.id)
        .await?
        .expect_one()?
        .length_seconds;

      Ok(Some(starts_at + Duration::seconds(length_seconds.into())))
    } else {
      Ok(None)
    }
  }

  async fn event(&self, ctx: &Context<'_>) -> Result<EventType, Error> {
    let query_data = ctx.data::<QueryData>()?;

    Ok(EventType::new(
      query_data
        .loaders()
        .run_event()
        .load_one(self.model.id)
        .await?
        .expect_one()?
        .clone(),
    ))
  }

  #[graphql(name = "my_signups")]
  async fn my_signups(&self, ctx: &Context<'_>) -> Result<Vec<SignupType>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    if let Some(user_con_profile) = query_data.user_con_profile() {
      let loader = query_data
        .loaders()
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
    if let Some(user_con_profile) = query_data.user_con_profile() {
      let loader = query_data
        .loaders()
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
  async fn not_counted_signup_count(&self, ctx: &Context<'_>) -> Result<i64, Error> {
    let query_data = ctx.data::<QueryData>()?;

    let counts = query_data
      .loaders()
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
        .loaders()
        .run_rooms()
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
      .loaders()
      .run_signup_counts
      .load_one(self.model.id)
      .await?
      .unwrap_or_default();

    Ok(JsonScalar(serde_json::to_value(
      counts.count_by_state_and_bucket_key_and_counted,
    )?))
  }

  #[graphql(name = "signups_paginated")]
  async fn signups_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    #[graphql(name = "per_page")] per_page: Option<u64>,
    filters: Option<SignupFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<SignupsPaginationType, Error> {
    let mut scope = self.model.find_related(signups::Entity);

    if let Some(filters) = filters {
      scope = filters.apply_filters(ctx, &scope)?;
    }

    if let Some(sort) = sort {
      for sort_column in sort {
        let order = sort_column.query_order();

        scope = match sort_column.field.as_str() {
          "id" => scope.order_by(signups::Column::Id, order),
          "state" => scope.order_by(signups::Column::State, order),
          "name" => scope
            .join(
              JoinType::InnerJoin,
              signups::Relation::UserConProfiles.def(),
            )
            .order_by(
              SimpleExpr::FunctionCall(Func::lower(Expr::col(user_con_profiles::Column::LastName))),
              order.clone(),
            )
            .order_by(
              SimpleExpr::FunctionCall(Func::lower(Expr::col(
                user_con_profiles::Column::FirstName,
              ))),
              order,
            ),
          "event_title" => scope
            .join(JoinType::InnerJoin, signups::Relation::Runs.def())
            .join(JoinType::InnerJoin, runs::Relation::Events.def())
            .order_by(
              SimpleExpr::FunctionCall(Func::lower(Expr::col(events::Column::Title))),
              order,
            ),
          "bucket" => scope.order_by(signups::Column::BucketKey, order),
          "email" => scope
            .join(
              JoinType::InnerJoin,
              signups::Relation::UserConProfiles.def(),
            )
            .join(
              JoinType::InnerJoin,
              user_con_profiles::Relation::Users.def(),
            )
            .order_by(users::Column::Email, order),
          "created_at" => scope.order_by(signups::Column::CreatedAt, order),
          _ => scope,
        }
      }
    }

    Ok(SignupsPaginationType::new(Some(scope), page, per_page))
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
