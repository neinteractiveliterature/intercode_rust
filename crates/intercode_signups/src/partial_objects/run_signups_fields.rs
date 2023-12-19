use std::sync::Arc;

use async_graphql::{Context, Error};
use axum::async_trait;
use intercode_entities::{runs, signup_changes, signup_requests, signups};
use intercode_graphql_core::{query_data::QueryData, ModelBackedType, ModelPaginator};
use intercode_graphql_loaders::LoaderManager;
use intercode_policies::AuthorizedFromQueryBuilder;
use intercode_query_builders::{sort_input::SortInput, PaginationFromQueryBuilder};
use sea_orm::ModelTrait;

use crate::{
  policies::SignupChangePolicy,
  query_builders::{
    SignupChangeFiltersInput, SignupChangesQueryBuilder, SignupFiltersInput, SignupsQueryBuilder,
  },
};

#[async_trait]
pub trait RunSignupsExtensions
where
  Self: ModelBackedType<Model = runs::Model>,
{
  async fn my_signups<T: ModelBackedType<Model = signups::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<T>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    if let Some(user_con_profile) = query_data.user_con_profile() {
      let loader = ctx
        .data::<Arc<LoaderManager>>()?
        .run_user_con_profile_signups
        .get(user_con_profile.id)
        .await;

      Ok(
        loader
          .load_one(self.get_model().id)
          .await?
          .unwrap_or_default()
          .iter()
          .cloned()
          .map(T::new)
          .collect(),
      )
    } else {
      Ok(vec![])
    }
  }

  async fn my_signup_requests<T: ModelBackedType<Model = signup_requests::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<T>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    if let Some(user_con_profile) = query_data.user_con_profile() {
      let loader = ctx
        .data::<Arc<LoaderManager>>()?
        .run_user_con_profile_signup_requests
        .get(user_con_profile.id)
        .await;

      Ok(
        loader
          .load_one(self.get_model().id)
          .await?
          .unwrap_or_default()
          .iter()
          .cloned()
          .map(T::new)
          .collect(),
      )
    } else {
      Ok(vec![])
    }
  }

  fn signup_changes_paginated<T: ModelBackedType<Model = signup_changes::Model>>(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    per_page: Option<u64>,
    filters: Option<SignupChangeFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<ModelPaginator<T>, Error> {
    ModelPaginator::authorized_from_query_builder(
      &SignupChangesQueryBuilder::new(filters, sort),
      ctx,
      self.get_model().find_related(signup_changes::Entity),
      page,
      per_page,
      SignupChangePolicy,
    )
  }

  fn signups_paginated<T: ModelBackedType<Model = signups::Model>>(
    &self,
    page: Option<u64>,
    per_page: Option<u64>,
    filters: Option<SignupFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> ModelPaginator<T> {
    ModelPaginator::from_query_builder(
      &SignupsQueryBuilder::new(filters, sort),
      self.get_model().find_related(signups::Entity),
      page,
      per_page,
    )
  }
}
