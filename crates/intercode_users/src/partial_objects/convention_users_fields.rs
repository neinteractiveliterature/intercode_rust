use async_graphql::*;
use intercode_entities::{
  conventions, links::ConventionToStaffPositions, model_ext::user_con_profiles::BioEligibility,
  staff_positions, user_con_profiles,
};
use intercode_graphql_core::{
  load_one_by_model_id, loader_result_to_many, loader_result_to_optional_single, model_backed_type,
  query_data::QueryData, ModelBackedType, ModelPaginator,
};
use intercode_policies::{policies::UserConProfilePolicy, AuthorizedFromQueryBuilder};
use intercode_query_builders::sort_input::SortInput;
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter};

use crate::query_builders::{UserConProfileFiltersInput, UserConProfilesQueryBuilder};

use super::{StaffPositionUsersFields, UserConProfileUsersFields};

model_backed_type!(ConventionUsersFields, conventions::Model);

impl ConventionUsersFields {
  pub async fn bio_eligible_user_con_profiles(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<UserConProfileUsersFields>, Error> {
    let db = ctx.data::<QueryData>()?.db();

    let profiles: Vec<UserConProfileUsersFields> = self
      .model
      .find_related(user_con_profiles::Entity)
      .bio_eligible()
      .all(db.as_ref())
      .await?
      .into_iter()
      .map(UserConProfileUsersFields::new)
      .collect::<Vec<_>>();

    Ok(profiles)
  }

  pub async fn catch_all_staff_position(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<StaffPositionUsersFields>> {
    let loader_result = load_one_by_model_id!(convention_catch_all_staff_position, ctx, self)?;
    Ok(loader_result_to_optional_single!(
      loader_result,
      StaffPositionUsersFields
    ))
  }

  pub async fn my_profile(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<UserConProfileUsersFields>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    let convention_id = query_data.convention().map(|c| c.id);

    if convention_id == Some(self.model.id) {
      Ok(
        query_data
          .user_con_profile()
          .cloned()
          .map(UserConProfileUsersFields::new),
      )
    } else if let Some(user) = query_data.current_user() {
      user_con_profiles::Entity::find()
        .filter(
          user_con_profiles::Column::ConventionId
            .eq(self.model.id)
            .and(user_con_profiles::Column::UserId.eq(user.id)),
        )
        .one(query_data.db())
        .await
        .map(|result| result.map(UserConProfileUsersFields::new))
        .map_err(|e| async_graphql::Error::new(e.to_string()))
    } else {
      Ok(None)
    }
  }

  pub async fn staff_position(
    &self,
    ctx: &Context<'_>,
    id: ID,
  ) -> Result<StaffPositionUsersFields, Error> {
    let db = ctx.data::<QueryData>()?.db();

    self
      .model
      .find_linked(ConventionToStaffPositions)
      .filter(staff_positions::Column::Id.eq(id.parse::<u64>()?))
      .one(db.as_ref())
      .await?
      .ok_or_else(|| {
        Error::new(format!(
          "Staff position with ID {} not found in convention",
          id.as_str()
        ))
      })
      .map(StaffPositionUsersFields::new)
  }

  pub async fn staff_positions(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<StaffPositionUsersFields>, Error> {
    let loader_result = load_one_by_model_id!(convention_staff_positions, ctx, self)?;
    Ok(loader_result_to_many!(
      loader_result,
      StaffPositionUsersFields
    ))
  }

  pub async fn user_con_profile(
    &self,
    ctx: &Context<'_>,
    id: ID,
  ) -> Result<UserConProfileUsersFields, Error> {
    let db = ctx.data::<QueryData>()?.db();

    self
      .model
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
      .map(UserConProfileUsersFields::new)
  }

  pub async fn user_con_profiles_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    per_page: Option<u64>,
    filters: Option<UserConProfileFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<ModelPaginator<UserConProfileUsersFields>, Error> {
    ModelPaginator::authorized_from_query_builder(
      &UserConProfilesQueryBuilder::new(filters, sort),
      ctx,
      self.model.find_related(user_con_profiles::Entity),
      page,
      per_page,
      UserConProfilePolicy,
    )
  }
}
