use cached::{async_sync::Mutex, CachedAsync, UnboundCache};
use intercode_entities::{signups, user_con_profiles, users};
use oxide_auth::endpoint::Scope;
use sea_orm::DbErr;
use seawater::ConnectionWrapper;
use std::{
  collections::{HashMap, HashSet},
  fmt::Debug,
  hash::Hash,
  sync::Arc,
};

use crate::{
  load_all_active_signups_in_convention_by_event_id, load_all_team_member_event_ids_in_convention,
  permissions_loading::{
    load_all_permissions_in_convention_with_model_type_and_id, UserPermissionsMap,
  },
};

pub type SignupsByEventId = HashMap<i64, Vec<signups::Model>>;

#[derive(Clone, Debug)]
pub struct AuthorizationInfo {
  pub db: ConnectionWrapper,
  pub user: Arc<Option<users::Model>>,
  pub oauth_scope: Option<Scope>,
  pub assumed_identity_from_profile: Arc<Option<user_con_profiles::Model>>,
  active_signups_by_convention_and_event: Arc<Mutex<UnboundCache<i64, SignupsByEventId>>>,
  all_model_permissions_by_convention: Arc<Mutex<UnboundCache<i64, UserPermissionsMap>>>,
  team_member_event_ids_by_convention_id: Arc<Mutex<UnboundCache<i64, HashSet<i64>>>>,
}

impl Hash for AuthorizationInfo {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.user.as_ref().as_ref().map(|u| u.id).hash(state);
    self.oauth_scope.as_ref().map(|s| s.to_string()).hash(state);
    self
      .assumed_identity_from_profile
      .as_ref()
      .as_ref()
      .map(|p| p.id)
      .hash(state);
  }
}

impl PartialEq for AuthorizationInfo {
  fn eq(&self, other: &Self) -> bool {
    self.user.as_ref().as_ref().map(|u| u.id) == other.user.as_ref().as_ref().map(|u| u.id)
      && self.oauth_scope == other.oauth_scope
      && self
        .assumed_identity_from_profile
        .as_ref()
        .as_ref()
        .map(|p| p.id)
        == other
          .assumed_identity_from_profile
          .as_ref()
          .as_ref()
          .map(|p| p.id)
  }
}

impl Eq for AuthorizationInfo {}

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
      active_signups_by_convention_and_event: Arc::new(Mutex::new(UnboundCache::new())),
      all_model_permissions_by_convention: Arc::new(Mutex::new(UnboundCache::new())),
      team_member_event_ids_by_convention_id: Arc::new(Mutex::new(UnboundCache::new())),
    }
  }

  pub async fn active_signups_in_convention_by_event_id(
    &self,
    convention_id: i64,
  ) -> Result<SignupsByEventId, DbErr> {
    let mut lock = self.active_signups_by_convention_and_event.lock().await;

    let signups_by_event_id = lock
      .try_get_or_set_with(convention_id, || {
        let user_id = self.user.as_ref().as_ref().map(|user| user.id);
        load_all_active_signups_in_convention_by_event_id(&self.db, convention_id, user_id)
      })
      .await?;

    Ok(signups_by_event_id.clone())
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

  pub async fn team_member_event_ids_in_convention(
    &self,
    convention_id: i64,
  ) -> Result<HashSet<i64>, DbErr> {
    let mut lock = self.team_member_event_ids_by_convention_id.lock().await;

    let event_ids = lock
      .try_get_or_set_with(convention_id, || {
        let user_id = self.user.as_ref().as_ref().map(|user| user.id);
        load_all_team_member_event_ids_in_convention(&self.db, convention_id, user_id)
      })
      .await?;

    Ok(event_ids.clone())
  }

  pub fn has_scope(&self, scope: &str) -> bool {
    if let Some(my_scope) = &self.oauth_scope {
      my_scope >= &scope.parse::<Scope>().unwrap()
    } else {
      // If there is no OAuth scope, we're a cookied user and therefore scopes don't apply
      true
    }
  }

  pub fn can_act_in_convention(&self, convention_id: i64) -> bool {
    if let Some(identity_assumer) = self.assumed_identity_from_profile.as_ref().as_ref() {
      if identity_assumer.convention_id != convention_id {
        return false;
      }
    }

    true
  }

  pub async fn has_convention_permission(
    &self,
    permission: &str,
    convention_id: i64,
  ) -> Result<bool, DbErr> {
    if !self.can_act_in_convention(convention_id) {
      return Ok(false);
    }

    let perms = self
      .all_model_permissions_in_convention(convention_id)
      .await?;

    Ok(perms.has_convention_permission(convention_id, permission))
  }

  pub async fn has_event_category_permission(
    &self,
    permission: &str,
    convention_id: i64,
    event_category_id: i64,
  ) -> Result<bool, DbErr> {
    if !self.can_act_in_convention(convention_id) {
      return Ok(false);
    }

    let perms = self
      .all_model_permissions_in_convention(convention_id)
      .await?;

    Ok(perms.has_event_category_permission(event_category_id, permission))
  }

  pub async fn has_scope_and_convention_permission(
    &self,
    scope: &str,
    permission: &str,
    convention_id: i64,
  ) -> Result<bool, DbErr> {
    if self.has_scope(scope) {
      self
        .has_convention_permission(permission, convention_id)
        .await
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

  pub async fn team_member_in_convention(&self, convention_id: i64) -> Result<bool, DbErr> {
    Ok(
      !self
        .team_member_event_ids_in_convention(convention_id)
        .await?
        .is_empty(),
    )
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
