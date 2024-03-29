use async_graphql::*;
use axum::async_trait;
use intercode_entities::{event_proposals, user_con_profiles, users, UserNames};
use intercode_graphql_core::{
  load_one_by_model_id, loader_result_to_many, model_backed_type, ModelBackedType,
};

model_backed_type!(UserUsersFields, users::Model);

#[async_trait]
pub trait UserUsersExtensions
where
  Self: ModelBackedType<Model = users::Model>,
{
  async fn event_proposals<T: ModelBackedType<Model = event_proposals::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<T>> {
    let loader_result = load_one_by_model_id!(user_event_proposals, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, T))
  }

  async fn user_con_profiles<T: ModelBackedType<Model = user_con_profiles::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<T>> {
    let loader_result = load_one_by_model_id!(user_user_con_profiles, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, T))
  }
}

#[Object]
impl UserUsersFields {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn email(&self) -> &String {
    &self.model.email
  }

  #[graphql(name = "first_name")]
  async fn first_name(&self) -> &str {
    &self.model.first_name
  }

  #[graphql(name = "last_name")]
  async fn last_name(&self) -> &str {
    &self.model.last_name
  }

  async fn name(&self) -> String {
    self.model.name_without_nickname()
  }

  #[graphql(name = "name_inverted")]
  async fn name_inverted(&self) -> String {
    self.model.name_inverted()
  }

  async fn privileges(&self) -> Vec<String> {
    if self.model.site_admin.unwrap_or(false) {
      vec!["site_admin".to_string()]
    } else {
      vec![]
    }
  }
}
