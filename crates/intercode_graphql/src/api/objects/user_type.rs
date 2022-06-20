use crate::users;
use async_graphql::*;
use intercode_entities::UserNames;

use crate::model_backed_type;
model_backed_type!(UserType, users::Model);

#[Object]
impl UserType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn email(&self) -> &String {
    &self.model.email
  }

  #[graphql(name = "name")]
  async fn name(&self) -> String {
    self.model.name_without_nickname()
  }

  #[graphql(name = "name_inverted")]
  async fn name_inverted(&self) -> String {
    self.model.name_inverted()
  }
}
