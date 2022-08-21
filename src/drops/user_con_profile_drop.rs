use intercode_entities::{signups, user_con_profiles, users, UserNames};
use intercode_graphql::SchemaData;
use intercode_inflector::IntercodeInflector;
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
use sea_orm::ModelTrait;

use super::{DropError, SignupDrop, UserDrop};

#[liquid_drop_struct]
pub struct UserConProfileDrop {
  user_con_profile: user_con_profiles::Model,
  schema_data: SchemaData,
}

#[liquid_drop_impl]
impl UserConProfileDrop {
  pub fn new(user_con_profile: user_con_profiles::Model, schema_data: SchemaData) -> Self {
    UserConProfileDrop {
      user_con_profile,
      schema_data,
    }
  }

  pub fn id(&self) -> i64 {
    self.user_con_profile.id
  }

  fn first_name(&self) -> &str {
    self.user_con_profile.first_name.as_str()
  }

  fn last_name(&self) -> &str {
    self.user_con_profile.last_name.as_str()
  }

  fn name_without_nickname(&self) -> String {
    self.user_con_profile.name_without_nickname()
  }

  async fn privileges(&self) -> Result<Vec<String>, DropError> {
    let inflector = IntercodeInflector::new();

    Ok(
      self
        .caching_user()
        .await
        .get_inner()
        .unwrap()
        .privileges()
        .iter()
        .map(|priv_name| inflector.humanize(priv_name))
        .collect::<Vec<_>>(),
    )
  }

  async fn signups(&self) -> Result<Vec<SignupDrop>, DropError> {
    let signups = self
      .user_con_profile
      .find_related(signups::Entity)
      .all(self.schema_data.db.as_ref())
      .await?;

    Ok(signups.into_iter().map(SignupDrop::new).collect::<Vec<_>>())
  }

  async fn user(&self) -> Result<UserDrop, DropError> {
    let user = self
      .user_con_profile
      .find_related(users::Entity)
      .one(self.schema_data.db.as_ref())
      .await?
      .ok_or_else(|| DropError::ExpectedEntityNotFound("User".to_string()))?;

    Ok(UserDrop::new(user))
  }
}
