use std::sync::Arc;

use async_graphql::*;
use intercode_entities::oauth_applications;
use intercode_graphql_core::{query_data::QueryData, ModelBackedType};
use intercode_policies::AuthorizationInfo;
use sea_orm::{EntityTrait, PaginatorTrait};

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
