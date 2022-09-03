use intercode_entities::{
  links::StaffPositionToUserConProfiles, staff_positions, user_con_profiles,
};
use intercode_graphql::{
  loaders::{expect::ExpectModels, EntityLinkLoaderResult},
  SchemaData,
};
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
use sea_orm::PrimaryKeyToColumn;

use crate::drops::preloaders::Preloader;

use super::{preloaders::EntityLinkPreloader, DropError, UserConProfileDrop};

#[liquid_drop_struct]
pub struct StaffPositionDrop {
  schema_data: SchemaData,
  staff_position: staff_positions::Model,
}

#[liquid_drop_impl]
impl StaffPositionDrop {
  pub fn new(staff_position: staff_positions::Model, schema_data: SchemaData) -> Self {
    StaffPositionDrop {
      schema_data,
      staff_position,
    }
  }

  pub fn id(&self) -> i64 {
    self.staff_position.id
  }

  fn email(&self) -> Option<&str> {
    self.staff_position.email.as_deref()
  }

  fn email_link(&self) -> Option<String> {
    self
      .email()
      .map(|email| format!("<a href=\"mailto:{}\">{}</a>", email, email))
  }

  fn name(&self) -> Option<&str> {
    self.staff_position.name.as_deref()
  }

  pub fn user_con_profiles_preloader(
    schema_data: SchemaData,
  ) -> EntityLinkPreloader<
    staff_positions::Entity,
    StaffPositionToUserConProfiles,
    user_con_profiles::Entity,
    staff_positions::PrimaryKey,
    Self,
    Vec<UserConProfileDrop>,
  > {
    EntityLinkPreloader::new(
      staff_positions::PrimaryKey::Id.into_column(),
      StaffPositionToUserConProfiles,
      |drop: &Self| drop.id(),
      move |value: Option<
        &EntityLinkLoaderResult<staff_positions::Entity, user_con_profiles::Entity>,
      >| {
        let user_con_profiles = value.expect_models()?;
        Ok(
          user_con_profiles
            .iter()
            .map(|ucp| UserConProfileDrop::new(ucp.clone(), schema_data.clone()))
            .collect(),
        )
      },
      |cache| &cache.user_con_profiles,
    )
  }

  pub async fn preload_user_con_profiles(
    schema_data: SchemaData,
    drops: &[&StaffPositionDrop],
  ) -> Result<(), DropError> {
    let preloader = StaffPositionDrop::user_con_profiles_preloader(schema_data.clone());
    let preloader_result = preloader.preload(schema_data.db.as_ref(), drops).await?;
    let values = preloader_result.all_values_flat_unwrapped();
    UserConProfileDrop::preload_users_and_signups(
      schema_data.clone(),
      values.iter().collect::<Vec<_>>().as_slice(),
    )
    .await
  }

  async fn user_con_profiles(&self) -> Result<&Vec<UserConProfileDrop>, DropError> {
    StaffPositionDrop::preload_user_con_profiles(self.schema_data.clone(), &[self]).await?;
    Ok(
      self
        .drop_cache
        .user_con_profiles
        .get()
        .unwrap()
        .get_inner()
        .unwrap(),
    )
  }
}
