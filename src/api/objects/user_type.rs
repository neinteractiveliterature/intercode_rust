use crate::users;
use async_graphql::*;

use super::ModelBackedType;
pub struct UserType {
  model: users::Model,
}

impl ModelBackedType<users::Model> for UserType {
  fn new(model: users::Model) -> Self {
    UserType { model }
  }
}

#[Object]
impl UserType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn email(&self) -> &String {
    &self.model.email
  }
}
