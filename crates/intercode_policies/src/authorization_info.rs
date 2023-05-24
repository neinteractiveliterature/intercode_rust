use cached::{async_sync::Mutex, CachedAsync, UnboundCache};
use intercode_entities::{
  cms_content_groups, conventions, event_categories, events, signups, user_con_profiles, users,
};
use oxide_auth::endpoint::Scope;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, PaginatorTrait, QueryFilter, Select};
use seawater::ConnectionWrapper;
use std::{
  collections::{HashMap, HashSet},
  fmt::Debug,
  hash::Hash,
};
use tokio::sync::OnceCell;

use crate::{
  conventions_where_team_member, conventions_with_permission, conventions_with_permissions,
  event_categories_with_permission, events_where_team_member,
  load_all_active_signups_in_convention_by_event_id, load_all_team_member_event_ids_in_convention,
  permissions_loading::{
    load_all_permissions_in_convention_with_model_type_and_id, UserPermissionsMap,
  },
};

pub type SignupsByEventId = HashMap<i64, Vec<signups::Model>>;

#[derive(Debug)]
pub struct AuthorizationInfo {
  pub db: ConnectionWrapper,
  pub user: Option<users::Model>,
  pub oauth_scope: Option<Scope>,
  pub assumed_identity_from_profile: Option<user_con_profiles::Model>,
  active_signups_by_convention_and_event: Mutex<UnboundCache<i64, SignupsByEventId>>,
  all_model_permissions_by_convention: Mutex<UnboundCache<i64, UserPermissionsMap>>,
  organization_permissions_by_organization_id: OnceCell<HashMap<i64, HashSet<String>>>,
  user_con_profile_ids: OnceCell<HashSet<i64>>,
  team_member_event_ids_by_convention_id: Mutex<UnboundCache<i64, HashSet<i64>>>,
}

impl Clone for AuthorizationInfo {
  fn clone(&self) -> Self {
    Self {
      db: self.db.clone(),
      user: self.user.clone(),
      oauth_scope: self.oauth_scope.clone(),
      assumed_identity_from_profile: self.assumed_identity_from_profile.clone(),
      active_signups_by_convention_and_event: Mutex::new(UnboundCache::new()),
      all_model_permissions_by_convention: Mutex::new(UnboundCache::new()),
      organization_permissions_by_organization_id: OnceCell::new(),
      user_con_profile_ids: OnceCell::new(),
      team_member_event_ids_by_convention_id: Mutex::new(UnboundCache::new()),
    }
  }
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
    user: Option<users::Model>,
    oauth_scope: Option<Scope>,
    assumed_identity_from_profile: Option<user_con_profiles::Model>,
  ) -> Self {
    Self {
      db,
      user,
      oauth_scope,
      assumed_identity_from_profile,
      active_signups_by_convention_and_event: Mutex::new(UnboundCache::new()),
      all_model_permissions_by_convention: Mutex::new(UnboundCache::new()),
      organization_permissions_by_organization_id: OnceCell::new(),
      user_con_profile_ids: OnceCell::new(),
      team_member_event_ids_by_convention_id: Mutex::new(UnboundCache::new()),
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
        load_all_permissions_in_convention_with_model_type_and_id(
          &self.db,
          Some(convention_id),
          user_id,
        )
      })
      .await?;

    Ok(permissions_map.clone())
  }

  pub async fn organization_permissions_by_organization_id(
    &self,
  ) -> Result<&HashMap<i64, HashSet<String>>, DbErr> {
    self
      .organization_permissions_by_organization_id
      .get_or_try_init(|| async {
        let user_id = self.user.as_ref().as_ref().map(|user| user.id);
        Ok(
          load_all_permissions_in_convention_with_model_type_and_id(&self.db, None, user_id)
            .await?
            .into_iter()
            .filter_map(|(model_id, permissions)| match model_id {
              crate::PermissionModelId::OrganizationId(organization_id) => {
                Some((organization_id, permissions))
              }
              _ => None,
            })
            .collect(),
        )
      })
      .await
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

  pub async fn user_con_profile_ids(&self) -> Result<&HashSet<i64>, DbErr> {
    self
      .user_con_profile_ids
      .get_or_try_init(|| async {
        let user_con_profile_ids = if let Some(user) = &self.user {
          let mut scope =
            user_con_profiles::Entity::find().filter(user_con_profiles::Column::UserId.eq(user.id));

          if let Some(assumed_identity_from_profile) = &self.assumed_identity_from_profile {
            scope = scope.filter(
              user_con_profiles::Column::ConventionId
                .eq(assumed_identity_from_profile.convention_id),
            );
          }

          scope
            .all(&self.db)
            .await?
            .into_iter()
            .map(|user_con_profile| user_con_profile.id)
            .collect()
        } else {
          vec![]
        };

        Ok(HashSet::from_iter(user_con_profile_ids.into_iter()))
      })
      .await
  }

  pub async fn cms_content_group_ids_with_permission_in_convention(
    &self,
    convention_id: i64,
    permission: &str,
  ) -> Result<HashSet<i64>, DbErr> {
    let perms = self
      .all_model_permissions_in_convention(convention_id)
      .await?;
    Ok(perms.cms_content_group_ids_with_permission(permission))
  }

  pub async fn cms_content_group_scope_has_permission(
    &self,
    scope: Select<cms_content_groups::Entity>,
    convention_id: i64,
    permission: &str,
  ) -> Result<bool, DbErr> {
    let group_ids_with_permission = self
      .cms_content_group_ids_with_permission_in_convention(convention_id, permission)
      .await?;

    Ok(
      scope
        .filter(cms_content_groups::Column::Id.is_in(group_ids_with_permission))
        .count(&self.db)
        .await?
        > 0,
    )
  }

  pub fn conventions_where_team_member(&self) -> Select<conventions::Entity> {
    conventions_where_team_member(self.user.as_ref().map(|user| user.id))
  }

  pub fn events_where_team_member(&self) -> Select<events::Entity> {
    events_where_team_member(self.user.as_ref().map(|user| user.id))
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

  pub fn conventions_with_permissions(&self, permissions: &[&str]) -> Select<conventions::Entity> {
    conventions_with_permissions(permissions, self.user.as_ref().map(|u| u.id))
  }

  pub fn conventions_with_permission(&self, permission: &str) -> Select<conventions::Entity> {
    conventions_with_permission(permission, self.user.as_ref().map(|u| u.id))
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

  pub fn event_categories_with_permission(
    &self,
    permission: &str,
  ) -> Select<event_categories::Entity> {
    event_categories_with_permission(permission, self.user.as_ref().map(|u| u.id))
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

  pub async fn has_event_category_permission_in_convention(
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

    Ok(perms.has_event_category_permission_in_convention(convention_id, permission))
  }

  pub async fn has_organization_permission(
    &self,
    permission: &str,
    organization_id: i64,
  ) -> Result<bool, DbErr> {
    let perms = self.organization_permissions_by_organization_id().await?;

    Ok(
      perms
        .get(&organization_id)
        .map(|org_perms| org_perms.contains(permission))
        .unwrap_or(false),
    )
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
    Self::new(db, user, oauth_scope, assumed_identity_from_profile)
  }
}
