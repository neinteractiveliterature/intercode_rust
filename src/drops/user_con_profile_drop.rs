use futures::try_join;
use intercode_entities::{signups, user_con_profiles, users, UserNames};
use intercode_graphql::{
  loaders::{expect::ExpectModels, EntityRelationLoaderResult},
  SchemaData,
};
use intercode_inflector::IntercodeInflector;
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct, DropResult};
use sea_orm::{ModelTrait, PrimaryKeyToColumn};

use crate::drops::preloaders::Preloader;

use super::{preloaders::EntityRelationPreloader, DropError, SignupDrop, UserDrop};

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

  pub fn signups_preloader() -> EntityRelationPreloader<
    user_con_profiles::Entity,
    signups::Entity,
    user_con_profiles::PrimaryKey,
    Self,
    Vec<SignupDrop>,
  > {
    EntityRelationPreloader::new(
      user_con_profiles::PrimaryKey::Id.into_column(),
      |drop: &Self| drop.id(),
      |result| {
        let signups: &Vec<signups::Model> = result.expect_models()?;
        Ok(
          signups
            .iter()
            .map(|signup| SignupDrop::new(signup.clone()))
            .collect(),
        )
      },
      |drop: &Self, value: DropResult<Vec<SignupDrop>>| {
        drop.drop_cache.set_signups(value).map_err(|err| err.into())
      },
    )
  }

  pub fn users_preloader() -> EntityRelationPreloader<
    user_con_profiles::Entity,
    users::Entity,
    user_con_profiles::PrimaryKey,
    Self,
    UserDrop,
  > {
    EntityRelationPreloader::new(
      user_con_profiles::PrimaryKey::Id.into_column(),
      |drop: &Self| drop.id(),
      |result: Option<&EntityRelationLoaderResult<user_con_profiles::Entity, users::Entity>>| {
        let user = result.expect_one()?;
        Ok(UserDrop::new(user.clone()))
      },
      |drop: &Self, value: DropResult<UserDrop>| {
        drop.drop_cache.set_user(value).map_err(|err| err.into())
      },
    )
  }

  pub async fn preload_users_and_signups(
    schema_data: SchemaData,
    drops: &[&UserConProfileDrop],
  ) -> Result<(), DropError> {
    try_join!(
      async {
        UserConProfileDrop::users_preloader()
          .preload(schema_data.db.as_ref(), drops)
          .await?;
        Ok::<(), DropError>(())
      },
      async {
        UserConProfileDrop::signups_preloader()
          .preload(schema_data.db.as_ref(), drops)
          .await?;
        Ok::<(), DropError>(())
      }
    )?;

    Ok(())
  }
}
