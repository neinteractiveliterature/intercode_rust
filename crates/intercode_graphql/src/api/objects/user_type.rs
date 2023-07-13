use crate::users;
use async_graphql::*;
use intercode_entities::UserNames;
use intercode_graphql_core::model_backed_type;

model_backed_type!(UserType, users::Model);

#[Object(name = "User")]
impl UserType {
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
}
