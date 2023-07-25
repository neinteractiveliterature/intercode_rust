use std::sync::Arc;

use async_graphql::*;
use intercode_entities::{events, runs, signups, user_con_profiles, users};
use intercode_events::partial_objects::RunEventsFields;
use intercode_graphql_core::{
  model_backed_type, query_data::QueryData, ModelBackedType, ModelPaginator,
  PaginationImplementation,
};
use intercode_graphql_loaders::LoaderManager;
use intercode_query_builders::sort_input::SortInput;
use sea_orm::{
  sea_query::{Expr, Func, SimpleExpr},
  JoinType, ModelTrait, QueryOrder, QuerySelect, RelationTrait,
};

use crate::{
  api::{
    inputs::SignupFiltersInput,
    objects::{SignupRequestType, SignupType},
  },
  merged_model_backed_type,
};

use super::EventType;

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

  #[graphql(name = "signups_paginated")]
  async fn signups_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    #[graphql(name = "per_page")] per_page: Option<u64>,
    filters: Option<SignupFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<ModelPaginator<SignupType>, Error> {
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

    Ok(ModelPaginator::new(Some(scope), page, per_page))
  }
}

merged_model_backed_type!(RunType, runs::Model, "Run", RunGlueFields, RunEventsFields);
