use std::sync::Arc;

use async_graphql::*;
use intercode_entities::{oauth_applications, users};
use intercode_graphql_core::{
  lax_id::LaxId, query_data::QueryData, ModelBackedType, ModelPaginator,
};
use intercode_policies::{AuthorizationInfo, AuthorizedFromQueryBuilder};
use intercode_query_builders::sort_input::SortInput;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};

use crate::{
  policies::UserPolicy,
  query_builders::{UserFiltersInput, UsersQueryBuilder},
};

use super::{AbilityUsersFields, UserConProfileUsersFields, UserUsersFields};

#[derive(Default)]
pub struct QueryRootUsersFields;

impl QueryRootUsersFields {
  pub async fn assumed_identity_from_profile(
    ctx: &Context<'_>,
  ) -> Result<Option<UserConProfileUsersFields>> {
    Ok(
      ctx
        .data::<AuthorizationInfo>()?
        .assumed_identity_from_profile
        .as_ref()
        .map(|profile| UserConProfileUsersFields::new(profile.clone())),
    )
  }

  pub async fn current_ability(ctx: &Context<'_>) -> Result<AbilityUsersFields> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    Ok(AbilityUsersFields::new(Arc::new(
      authorization_info.clone(),
    )))
  }

  pub async fn current_user(ctx: &Context<'_>) -> Result<Option<UserUsersFields>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    Ok(query_data.current_user().cloned().map(UserUsersFields::new))
  }

  pub async fn user(ctx: &Context<'_>, id: Option<ID>) -> Result<UserUsersFields, Error> {
    let query_data = ctx.data::<QueryData>()?;
    let user = users::Entity::find_by_id(LaxId::parse(id.clone().unwrap_or_default())?)
      .one(query_data.db())
      .await?
      .ok_or_else(|| Error::new(format!("User {} not found", id.unwrap_or_default().0)))?;

    Ok(UserUsersFields::new(user))
  }

  pub async fn users(
    ctx: &Context<'_>,
    ids: Option<Vec<ID>>,
  ) -> Result<Vec<UserUsersFields>, Error> {
    let ids = ids.unwrap_or_default();

    if ids.len() > 25 {
      return Err(Error::new(
        "Can't retrieve more than 25 users in a single query",
      ));
    }

    let query_data = ctx.data::<QueryData>()?;
    Ok(
      users::Entity::find()
        .filter(
          users::Column::Id.is_in(
            ids
              .into_iter()
              .map(LaxId::parse)
              .collect::<Result<Vec<_>, _>>()?,
          ),
        )
        .all(query_data.db())
        .await?
        .into_iter()
        .map(UserUsersFields::new)
        .collect(),
    )
  }

  pub async fn users_paginated(
    ctx: &Context<'_>,
    page: Option<u64>,
    per_page: Option<u64>,
    filters: Option<UserFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<ModelPaginator<UserUsersFields>, Error> {
    ModelPaginator::authorized_from_query_builder(
      &UsersQueryBuilder::new(filters, sort),
      ctx,
      users::Entity::find(),
      page,
      per_page,
      UserPolicy,
    )
  }
}

#[Object]
impl QueryRootUsersFields {
  async fn has_oauth_applications(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    let query_data = ctx.data::<QueryData>()?;

    let count = oauth_applications::Entity::find()
      .count(query_data.db())
      .await?;
    Ok(count > 0)
  }
}
