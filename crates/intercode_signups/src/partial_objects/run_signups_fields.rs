use std::sync::Arc;

use async_graphql::{Context, Error};
use intercode_entities::{runs, signups};
use intercode_graphql_core::{
  model_backed_type, query_data::QueryData, ModelBackedType, ModelPaginator,
};
use intercode_graphql_loaders::LoaderManager;
use intercode_query_builders::{sort_input::SortInput, PaginationFromQueryBuilder};
use sea_orm::ModelTrait;

use crate::query_builders::{SignupFiltersInput, SignupsQueryBuilder};

use super::{SignupRequestSignupsFields, SignupSignupsFields};

model_backed_type!(RunSignupsFields, runs::Model);

impl RunSignupsFields {
  pub async fn my_signups(&self, ctx: &Context<'_>) -> Result<Vec<SignupSignupsFields>, Error> {
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
          .map(SignupSignupsFields::new)
          .collect(),
      )
    } else {
      Ok(vec![])
    }
  }

  pub async fn my_signup_requests(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<SignupRequestSignupsFields>, Error> {
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
          .map(SignupRequestSignupsFields::new)
          .collect(),
      )
    } else {
      Ok(vec![])
    }
  }

  pub fn signups_paginated(
    &self,
    page: Option<u64>,
    per_page: Option<u64>,
    filters: Option<SignupFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> ModelPaginator<SignupSignupsFields> {
    ModelPaginator::from_query_builder(
      &SignupsQueryBuilder::new(filters, sort),
      self.model.find_related(signups::Entity),
      page,
      per_page,
    )
  }
}
