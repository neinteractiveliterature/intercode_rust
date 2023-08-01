use async_graphql::*;
use intercode_entities::{permissions, staff_positions};
use intercode_graphql_core::{load_one_by_model_id, loader_result_to_many, model_backed_type};
use seawater::loaders::ExpectModels;

use super::UserConProfileUsersFields;

model_backed_type!(StaffPositionUsersFields, staff_positions::Model);

impl StaffPositionUsersFields {
  pub async fn permissions(&self, ctx: &Context<'_>) -> Result<Vec<permissions::Model>> {
    let loader_result = load_one_by_model_id!(staff_position_permissions, ctx, self)?;
    loader_result.expect_models().cloned()
  }

  pub async fn user_con_profiles(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<UserConProfileUsersFields>> {
    let loader_result = load_one_by_model_id!(staff_position_user_con_profiles, ctx, self)?;
    Ok(loader_result_to_many!(
      loader_result,
      UserConProfileUsersFields
    ))
  }
}

#[Object]
impl StaffPositionUsersFields {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "cc_addresses")]
  async fn cc_addresses(&self) -> &Vec<String> {
    &self.model.cc_addresses
  }

  async fn email(&self) -> Option<&str> {
    self.model.email.as_deref()
  }

  #[graphql(name = "email_aliases")]
  async fn email_aliases(&self) -> &Vec<String> {
    &self.model.email_aliases
  }

  async fn name(&self) -> Option<&str> {
    self.model.name.as_deref()
  }

  async fn visible(&self) -> bool {
    self.model.visible.unwrap_or(false)
  }
}
