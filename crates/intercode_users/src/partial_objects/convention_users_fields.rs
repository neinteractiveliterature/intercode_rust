use async_graphql::*;
use axum::async_trait;
use intercode_entities::{
  conventions, links::ConventionToStaffPositions, model_ext::user_con_profiles::BioEligibility,
  staff_positions, user_con_profiles,
};
use intercode_graphql_core::{
  lax_id::LaxId, load_one_by_model_id, loader_result_to_many, loader_result_to_optional_single,
  model_backed_type, query_data::QueryData, ModelBackedType, ModelPaginator,
};
use intercode_policies::{policies::UserConProfilePolicy, AuthorizedFromQueryBuilder};
use intercode_query_builders::sort_input::SortInput;
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter};

use crate::query_builders::{UserConProfileFiltersInput, UserConProfilesQueryBuilder};

model_backed_type!(ConventionUsersFields, conventions::Model);

#[async_trait]
pub trait ConventionUsersExtensions
where
  Self: ModelBackedType<Model = conventions::Model>,
{
  async fn bio_eligible_user_con_profiles<T: ModelBackedType<Model = user_con_profiles::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<T>, Error> {
    let db = ctx.data::<QueryData>()?.db();

    let profiles: Vec<T> = self
      .get_model()
      .find_related(user_con_profiles::Entity)
      .bio_eligible()
      .all(db.as_ref())
      .await?
      .into_iter()
      .map(T::new)
      .collect::<Vec<_>>();

    Ok(profiles)
  }

  async fn catch_all_staff_position<T: ModelBackedType<Model = staff_positions::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<T>> {
    let loader_result = load_one_by_model_id!(convention_catch_all_staff_position, ctx, self)?;
    Ok(loader_result_to_optional_single!(loader_result, T))
  }

  async fn my_profile<T: ModelBackedType<Model = user_con_profiles::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<T>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    let convention_id = query_data.convention().map(|c| c.id);

    if convention_id == Some(self.get_model().id) {
      Ok(query_data.user_con_profile().cloned().map(T::new))
    } else if let Some(user) = query_data.current_user() {
      user_con_profiles::Entity::find()
        .filter(
          user_con_profiles::Column::ConventionId
            .eq(self.get_model().id)
            .and(user_con_profiles::Column::UserId.eq(user.id)),
        )
        .one(query_data.db())
        .await
        .map(|result| result.map(T::new))
        .map_err(|e| async_graphql::Error::new(e.to_string()))
    } else {
      Ok(None)
    }
  }

  async fn staff_position<T: ModelBackedType<Model = staff_positions::Model>>(
    &self,
    ctx: &Context<'_>,
    id: Option<ID>,
  ) -> Result<T, Error> {
    let db = ctx.data::<QueryData>()?.db();

    self
      .get_model()
      .find_linked(ConventionToStaffPositions)
      .filter(staff_positions::Column::Id.eq(LaxId::parse(id.clone().unwrap_or_default())?))
      .one(db.as_ref())
      .await?
      .ok_or_else(|| {
        Error::new(format!(
          "Staff position with ID {} not found in convention",
          id.unwrap_or_default().as_str()
        ))
      })
      .map(T::new)
  }

  async fn staff_positions<T: ModelBackedType<Model = staff_positions::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<T>, Error> {
    let loader_result = load_one_by_model_id!(convention_staff_positions, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, T))
  }

  async fn user_con_profile<T: ModelBackedType<Model = user_con_profiles::Model>>(
    &self,
    ctx: &Context<'_>,
    id: ID,
  ) -> Result<T, Error> {
    let db = ctx.data::<QueryData>()?.db();

    self
      .get_model()
      .find_related(user_con_profiles::Entity)
      .filter(user_con_profiles::Column::Id.eq(id.parse::<u64>()?))
      .one(db.as_ref())
      .await?
      .ok_or_else(|| {
        Error::new(format!(
          "No user con profile with ID {} in convention",
          id.as_str()
        ))
      })
      .map(T::new)
  }

  async fn user_con_profile_by_user_id<T: ModelBackedType<Model = user_con_profiles::Model>>(
    &self,
    ctx: &Context<'_>,
    user_id: ID,
  ) -> Result<T, Error> {
    let db = ctx.data::<QueryData>()?.db();

    self
      .get_model()
      .find_related(user_con_profiles::Entity)
      .filter(user_con_profiles::Column::UserId.eq(user_id.parse::<u64>()?))
      .one(db.as_ref())
      .await?
      .ok_or_else(|| {
        Error::new(format!(
          "No user con profile with ID {} in convention",
          user_id.as_str()
        ))
      })
      .map(T::new)
  }

  async fn user_con_profiles_paginated<T: ModelBackedType<Model = user_con_profiles::Model>>(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    per_page: Option<u64>,
    filters: Option<UserConProfileFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<ModelPaginator<T>, Error> {
    ModelPaginator::authorized_from_query_builder(
      &UserConProfilesQueryBuilder::new(filters, sort),
      ctx,
      self.get_model().find_related(user_con_profiles::Entity),
      page,
      per_page,
      UserConProfilePolicy,
    )
  }
}
