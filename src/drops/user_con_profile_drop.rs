use intercode_entities::{user_con_profiles, UserNames};
use intercode_graphql::loaders::expect::ExpectModels;
use intercode_graphql::SchemaData;
use intercode_inflector::IntercodeInflector;
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};

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
    let result = self
      .schema_data
      .loaders
      .user_con_profile_signups
      .load_one(self.user_con_profile.id)
      .await?;
    let signups = result.expect_models()?;

    Ok(
      signups
        .iter()
        .map(|signup| SignupDrop::new(signup.clone()))
        .collect::<Vec<_>>(),
    )
  }

  async fn user(&self) -> Result<UserDrop, DropError> {
    let result = self
      .schema_data
      .loaders
      .user_con_profile_user
      .load_one(self.user_con_profile.id)
      .await?;
    let user = result.expect_one()?;
    Ok(UserDrop::new(user.clone()))
  }
}
