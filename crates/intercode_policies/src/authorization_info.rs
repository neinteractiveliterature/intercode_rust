use intercode_entities::{user_con_profiles, users};
use memo_map::MemoMap;
use oxide_auth::endpoint::Scope;
use sea_orm::DbErr;
use seawater::ConnectionWrapper;
use std::{fmt::Debug, sync::Arc};

use crate::permissions_loading::{
  load_all_permissions_in_convention_with_model_type_and_id, UserPermissionsMap,
};

#[derive(Clone, Debug)]
pub struct AuthorizationInfo {
  pub db: ConnectionWrapper,
  pub user: Arc<Option<users::Model>>,
  pub oauth_scope: Option<Scope>,
  pub assumed_identity_from_profile: Arc<Option<user_con_profiles::Model>>,
  all_model_permissions_by_convention: MemoMap<i64, UserPermissionsMap>,
}

impl AuthorizationInfo {
  pub fn new(
    db: ConnectionWrapper,
    user: Arc<Option<users::Model>>,
    oauth_scope: Option<Scope>,
    assumed_identity_from_profile: Arc<Option<user_con_profiles::Model>>,
  ) -> Self {
    Self {
      db,
      user,
      oauth_scope,
      assumed_identity_from_profile,
      all_model_permissions_by_convention: Default::default(),
    }
  }

  pub async fn all_model_permissions_in_convention(
    &self,
    convention_id: i64,
  ) -> Result<UserPermissionsMap, DbErr> {
    let user_id = self.user.as_ref().as_ref().map(|user| user.id);
    load_all_permissions_in_convention_with_model_type_and_id(&self.db, convention_id, user_id)
      .await
    // self
    //   .all_model_permissions_by_convention
    //   .get_or_try_insert(&convention_id, || {
    //     let user_id = self.user.as_ref().as_ref().map(|user| user.id);
    //     ::tokio::task::block_in_place(move || {
    //       ::tokio::runtime::Handle::current().block_on(
    //         load_all_permissions_in_convention_with_model_type_and_id(
    //           &self.db,
    //           convention_id,
    //           user_id,
    //         ),
    //       )
    //     })
    //   })
  }

  pub fn has_scope(&self, scope: &str) -> bool {
    if let Some(my_scope) = &self.oauth_scope {
      my_scope >= &scope.parse::<Scope>().unwrap()
    } else {
      // If there is no OAuth scope, we're a cookied user and therefore scopes don't apply
      true
    }
  }

  pub fn site_admin(&self) -> bool {
    self
      .user
      .as_ref()
      .as_ref()
      .and_then(|u| u.site_admin)
      .unwrap_or(false)
  }

  pub fn site_admin_read(&self) -> bool {
    self.site_admin() && self.has_scope("read_conventions")
  }

  pub fn site_admin_manage(&self) -> bool {
    self.site_admin() && self.has_scope("manage_conventions")
  }
}

#[cfg(test)]
impl AuthorizationInfo {
  pub async fn for_test(
    db: ConnectionWrapper,
    user: Option<users::Model>,
    oauth_scope: Option<Scope>,
    assumed_identity_from_profile: Option<user_con_profiles::Model>,
  ) -> Self {
    Self::new(
      db,
      Arc::new(user),
      oauth_scope,
      Arc::new(assumed_identity_from_profile),
    )
  }
}
