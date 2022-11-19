use cached::{async_sync::Mutex, CachedAsync, UnboundCache};
use intercode_entities::{user_con_profiles, users};
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
  all_model_permissions_by_convention: Arc<Mutex<UnboundCache<i64, UserPermissionsMap>>>,
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
      all_model_permissions_by_convention: Arc::new(Mutex::new(UnboundCache::new())),
    }
  }

  pub async fn all_model_permissions_in_convention(
    &self,
    convention_id: i64,
  ) -> Result<UserPermissionsMap, DbErr> {
    let mut lock = self.all_model_permissions_by_convention.lock().await;

    let permissions_map = lock
      .try_get_or_set_with(convention_id, || {
        let user_id = self.user.as_ref().as_ref().map(|user| user.id);
        load_all_permissions_in_convention_with_model_type_and_id(&self.db, convention_id, user_id)
      })
      .await?;

    Ok(permissions_map.clone())
  }

  pub fn has_scope(&self, scope: &str) -> bool {
    if let Some(my_scope) = &self.oauth_scope {
      my_scope >= &scope.parse::<Scope>().unwrap()
    } else {
      // If there is no OAuth scope, we're a cookied user and therefore scopes don't apply
      true
    }
  }

  pub async fn has_scope_and_convention_permission(
    &self,
    scope: &str,
    permission: &str,
    convention_id: i64,
  ) -> Result<bool, DbErr> {
    if self.has_scope(scope) {
      let perms = self
        .all_model_permissions_in_convention(convention_id)
        .await?;

      Ok(perms.has_convention_permission(convention_id, permission))
    } else {
      Ok(false)
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
