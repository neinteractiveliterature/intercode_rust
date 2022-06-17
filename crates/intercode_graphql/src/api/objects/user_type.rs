use crate::users;
use async_graphql::*;

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
}
