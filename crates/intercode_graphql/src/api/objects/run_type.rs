use std::sync::Arc;

use async_graphql::{Context, Error, Object, Result, ID};
use chrono::Duration;
use intercode_entities::{events, runs, signups, user_con_profiles, users};
use intercode_graphql_core::{
  load_one_by_model_id, loader_result_to_many, loader_result_to_required_single, model_backed_type,
  scalars::{DateScalar, JsonScalar},
  ModelBackedType,
};
use intercode_graphql_loaders::LoaderManager;
use intercode_policies::{
  policies::{RunAction, RunPolicy},
  AuthorizationInfo, Policy,
};
use intercode_query_builders::sort_input::SortInput;
use sea_orm::{
  sea_query::{Expr, Func, SimpleExpr},
  JoinType, ModelTrait, QueryOrder, QuerySelect, RelationTrait,
};
use seawater::loaders::ExpectModel;

use crate::{
  api::{inputs::SignupFiltersInput, interfaces::PaginationImplementation},
  QueryData,
};

use super::{
  signup_request_type::SignupRequestType, EventType, RoomType, SignupType, SignupsPaginationType,
};

model_backed_type!(RunType, runs::Model);

#[Object(name = "Run")]
impl RunType {
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
      &(convention, event, self.model.clone()),
    )
    .await
    .map_err(|err| err.into())
  }

  #[graphql(name = "ends_at")]
  async fn ends_at(&self, ctx: &Context<'_>) -> Result<Option<DateScalar>, Error> {
    let starts_at = self.model.starts_at;

    if let Some(starts_at) = starts_at {
      let length_seconds = ctx
        .data::<Arc<LoaderManager>>()?
        .run_event()
        .load_one(self.model.id)
        .await?
        .expect_one()?
        .length_seconds;

      (starts_at + Duration::seconds(length_seconds.into()))
        .try_into()
        .map(Some)
    } else {
      Ok(None)
    }
  }

  async fn event(&self, ctx: &Context<'_>) -> Result<EventType, Error> {
    let loader_result = load_one_by_model_id!(run_event, ctx, self)?;
    Ok(loader_result_to_required_single!(loader_result, EventType))
  }

  #[graphql(name = "my_signups")]
  async fn my_signups(&self, ctx: &Context<'_>) -> Result<Vec<SignupType>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    if let Some(user_con_profile) = query_data.user_con_profile() {
      let loader = ctx
        .data::<Arc<LoaderManager>>()?
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
      let loader = ctx
        .data::<Arc<LoaderManager>>()?
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
    let loader_result = load_one_by_model_id!(run_rooms, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, RoomType))
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
  async fn starts_at(&self) -> Result<Option<DateScalar>> {
    self.model.starts_at.map(DateScalar::try_from).transpose()
  }

  #[graphql(name = "title_suffix")]
  async fn title_suffix(&self) -> Option<&str> {
    self.model.title_suffix.as_deref()
  }
}
