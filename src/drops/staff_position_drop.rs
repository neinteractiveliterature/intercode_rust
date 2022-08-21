use std::collections::HashMap;

use futures::join;
use intercode_entities::{
  links::StaffPositionToUserConProfiles, signups, staff_positions, user_con_profiles, users,
};
use intercode_graphql::{
  loaders::{expect::ExpectModels, load_all_linked, load_all_related},
  SchemaData,
};
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
use sea_orm::PrimaryKeyToColumn;

use super::{DropError, SignupDrop, UserConProfileDrop, UserDrop};

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

  pub async fn preload_user_con_profiles(
    schema_data: &SchemaData,
    drops: &[&StaffPositionDrop],
  ) -> Result<(), DropError> {
    let user_con_profile_lists = load_all_linked(
      staff_positions::PrimaryKey::Id.into_column(),
      &drops.iter().map(|drop| drop.id()).collect::<Vec<_>>(),
      &StaffPositionToUserConProfiles,
      schema_data.db.as_ref(),
    )
    .await?;

    let mut user_con_profile_drops_by_staff_position_id: HashMap<i64, Vec<UserConProfileDrop>> =
      Default::default();

    for drop in drops {
      let result = user_con_profile_lists.get(&drop.id());
      let user_con_profiles = result.expect_models()?;

      let user_con_profile_drops = user_con_profiles
        .iter()
        .map(|ucp| UserConProfileDrop::new(ucp.clone(), schema_data.clone()))
        .collect::<Vec<_>>();
      user_con_profile_drops_by_staff_position_id.insert(drop.id(), user_con_profile_drops);
    }

    let user_con_profile_ids = user_con_profile_drops_by_staff_position_id
      .values()
      .flat_map(|user_con_profile_drops| user_con_profile_drops.iter().map(|ucp| ucp.id()))
      .collect::<Vec<_>>();

    let (users, signups) = join!(
      load_all_related::<user_con_profiles::Entity, users::Entity, user_con_profiles::PrimaryKey>(
        user_con_profiles::PrimaryKey::Id.into_column(),
        &user_con_profile_ids,
        schema_data.db.as_ref(),
      ),
      load_all_related::<user_con_profiles::Entity, signups::Entity, user_con_profiles::PrimaryKey>(
        user_con_profiles::PrimaryKey::Id.into_column(),
        &user_con_profile_ids,
        schema_data.db.as_ref(),
      )
    );

    let users = users?;
    let signups = signups?;

    for ucp_drop in user_con_profile_drops_by_staff_position_id
      .values()
      .flatten()
    {
      let user = users.get(&ucp_drop.id()).unwrap().expect_one()?;
      let signups_result = signups.get(&ucp_drop.id());
      let signups = signups_result.expect_models()?;

      ucp_drop
        .drop_cache
        .set_user(UserDrop::new(user.clone()).into())?;

      ucp_drop.drop_cache.set_signups(
        signups
          .iter()
          .map(|signup| SignupDrop::new(signup.clone()))
          .collect::<Vec<_>>()
          .into(),
      )?;
    }

    for drop in drops {
      drop.drop_cache.set_user_con_profiles(
        user_con_profile_drops_by_staff_position_id
          .remove(&drop.id())
          .unwrap()
          .into(),
      )?;
    }

    Ok(())
  }

  async fn user_con_profiles(&self) -> Result<&Vec<UserConProfileDrop>, DropError> {
    StaffPositionDrop::preload_user_con_profiles(&self.schema_data, &[self]).await?;
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
