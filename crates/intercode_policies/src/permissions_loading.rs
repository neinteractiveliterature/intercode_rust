use std::collections::{HashMap, HashSet};

use intercode_entities::{
  cms_content_groups, event_categories, events, organization_roles_users, permissions, runs,
  signups, staff_positions, staff_positions_user_con_profiles, team_members, user_con_profiles,
};
use itertools::Itertools;
use sea_orm::{
  sea_query::{Alias, Expr},
  ColumnTrait, Condition, DbErr, EntityTrait, FromQueryResult, JoinType, QueryFilter, QuerySelect,
  RelationTrait, Select,
};
use seawater::ConnectionWrapper;

pub fn user_permission_scope(user_id: Option<i64>) -> Select<permissions::Entity> {
  match user_id {
    Some(user_id) => permissions::Entity::find().filter(
      Condition::any()
        .add(
          permissions::Column::StaffPositionId.in_subquery(
            sea_orm::QuerySelect::query(
              &mut staff_positions::Entity::find()
                .join(
                  JoinType::InnerJoin,
                  staff_positions_user_con_profiles::Relation::StaffPositions
                    .def()
                    .rev(),
                )
                .join_as(
                  JoinType::InnerJoin,
                  staff_positions_user_con_profiles::Relation::UserConProfiles.def(),
                  Alias::new("user_con_profiles"),
                )
                .filter(user_con_profiles::Column::UserId.eq(user_id))
                .select_only()
                .column(staff_positions::Column::Id),
            )
            .take(),
          ),
        )
        .add(
          permissions::Column::OrganizationRoleId.in_subquery(
            sea_orm::QuerySelect::query(
              &mut organization_roles_users::Entity::find()
                .filter(organization_roles_users::Column::UserId.eq(user_id))
                .select_only()
                .column(organization_roles_users::Column::OrganizationRoleId),
            )
            .take(),
          ),
        ),
    ),
    None => permissions::Entity::find().filter(Expr::cust("1 = 0")),
  }
}

pub fn select_all_permissions_in_convention(
  convention_id: i64,
  user_id: Option<i64>,
) -> Select<permissions::Entity> {
  user_permission_scope(user_id).filter(
    Condition::any()
      .add(permissions::Column::ConventionId.eq(convention_id))
      .add(
        permissions::Column::EventCategoryId.in_subquery(
          sea_orm::QuerySelect::query(
            &mut event_categories::Entity::find()
              .filter(event_categories::Column::ConventionId.eq(convention_id))
              .select_only()
              .column(event_categories::Column::Id),
          )
          .take(),
        ),
      )
      .add(
        permissions::Column::CmsContentGroupId.in_subquery(
          sea_orm::QuerySelect::query(
            &mut cms_content_groups::Entity::find()
              .filter(cms_content_groups::Column::ParentType.eq("Convention"))
              .filter(cms_content_groups::Column::ParentId.eq(convention_id))
              .select_only()
              .column(cms_content_groups::Column::Id),
          )
          .take(),
        ),
      ),
  )
}

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub enum PermissionModelId {
  Convention(i64),
  EventCategory(i64),
  CmsContentGroup(i64),
}

#[derive(Clone)]
pub struct UserPermission {
  pub permission: String,
  pub model_id: PermissionModelId,
  pub convention_id: Option<i64>,
}

impl FromQueryResult for UserPermission {
  fn from_query_result(res: &sea_orm::QueryResult, pre: &str) -> Result<Self, DbErr> {
    let permission: String = res.try_get(pre, "permission")?;
    let convention_id: Option<i64> = res.try_get(pre, "convention_id")?;
    let event_category_id: Option<i64> = res.try_get(pre, "event_category_id")?;
    let cms_content_group_id: Option<i64> = res.try_get(pre, "cms_content_group_id")?;
    let model_id = if let Some(convention_id) = convention_id {
      PermissionModelId::Convention(convention_id)
    } else if let Some(event_category_id) = event_category_id {
      PermissionModelId::EventCategory(event_category_id)
    } else if let Some(cms_content_group_id) = cms_content_group_id {
      PermissionModelId::CmsContentGroup(cms_content_group_id)
    } else {
      return Err(DbErr::Custom(
        "Permission record did not have a model ID".to_string(),
      ));
    };

    Ok(Self {
      permission,
      model_id,
      convention_id,
    })
  }
}

#[derive(Debug, Clone, Default)]
pub struct UserPermissionsMap {
  permissions: HashMap<PermissionModelId, HashSet<String>>,
  event_category_permissions_by_convention_id: HashMap<i64, HashSet<String>>,
}

impl UserPermissionsMap {
  pub fn from_user_permissions(iter: &mut dyn Iterator<Item = UserPermission>) -> Self {
    let (permissions_iter, event_category_permissions_iter) = iter.tee();
    let permissions = permissions_iter
      .map(|user_permission| (user_permission.model_id, user_permission.permission))
      .fold(HashMap::new(), |mut acc, (model_id, permission)| {
        let entry = acc.entry(model_id);
        let permissions: &mut HashSet<String> = entry.or_default();
        permissions.insert(permission);
        acc
      });

    let event_category_permissions_by_convention_id = event_category_permissions_iter
      .filter_map(|user_permission| match user_permission.model_id {
        PermissionModelId::EventCategory(_) => user_permission
          .convention_id
          .map(|convention_id| (convention_id, user_permission.permission)),
        _ => None,
      })
      .fold(HashMap::new(), |mut acc, (convention_id, permission)| {
        let entry = acc.entry(convention_id);
        let permissions: &mut HashSet<String> = entry.or_default();
        permissions.insert(permission);
        acc
      });

    Self {
      permissions,
      event_category_permissions_by_convention_id,
    }
  }

  pub fn has_permission(&self, model_id: &PermissionModelId, permission: &str) -> bool {
    self
      .permissions
      .get(model_id)
      .map(|perm_set| perm_set.contains(permission))
      .unwrap_or(false)
  }

  pub fn has_any_permission(&self, permission: &str) -> bool {
    self
      .permissions
      .values()
      .any(|perm_set| perm_set.contains(permission))
  }

  pub fn has_convention_permission(&self, convention_id: i64, permission: &str) -> bool {
    self.has_permission(&PermissionModelId::Convention(convention_id), permission)
  }

  pub fn has_event_category_permission(&self, event_category_id: i64, permission: &str) -> bool {
    self.has_permission(
      &PermissionModelId::EventCategory(event_category_id),
      permission,
    )
  }

  pub fn has_event_category_permission_in_convention(
    &self,
    convention_id: i64,
    permission: &str,
  ) -> bool {
    self
      .event_category_permissions_by_convention_id
      .get(&convention_id)
      .map(|perm_set| perm_set.contains(permission))
      .unwrap_or(false)
  }

  pub fn has_cms_content_group_permission(
    &self,
    cms_content_group_id: i64,
    permission: &str,
  ) -> bool {
    self.has_permission(
      &PermissionModelId::CmsContentGroup(cms_content_group_id),
      permission,
    )
  }

  pub fn cms_content_group_ids_with_permission(&self, permission: &str) -> HashSet<i64> {
    self
      .permissions
      .iter()
      .filter_map(
        |(permission_model_id, permissions)| match permission_model_id {
          PermissionModelId::CmsContentGroup(id) => {
            if permissions.contains(permission) {
              Some(id)
            } else {
              None
            }
          }
          _ => None,
        },
      )
      .copied()
      .collect()
  }
}

pub async fn load_all_permissions_in_convention_with_model_type_and_id(
  db: &ConnectionWrapper,
  convention_id: i64,
  user_id: Option<i64>,
) -> Result<UserPermissionsMap, DbErr> {
  Ok(UserPermissionsMap::from_user_permissions(
    &mut select_all_permissions_in_convention(convention_id, user_id)
      .select_only()
      .column(permissions::Column::Permission)
      .column(permissions::Column::ConventionId)
      .column(permissions::Column::EventCategoryId)
      .column(permissions::Column::CmsContentGroupId)
      .into_model::<UserPermission>()
      .all(db)
      .await?
      .into_iter(),
  ))
}

pub async fn load_all_active_signups_in_convention_by_event_id(
  db: &ConnectionWrapper,
  convention_id: i64,
  user_id: Option<i64>,
) -> Result<HashMap<i64, Vec<signups::Model>>, DbErr> {
  let signups_with_runs = signups::Entity::find()
    .join(
      JoinType::InnerJoin,
      signups::Relation::UserConProfiles.def(),
    )
    .join(JoinType::InnerJoin, signups::Relation::Runs.def())
    .join(JoinType::InnerJoin, runs::Relation::Events.def())
    .select_also(runs::Entity)
    .filter(user_con_profiles::Column::UserId.eq(user_id))
    .filter(events::Column::ConventionId.eq(convention_id))
    .all(db)
    .await?;

  Ok(
    signups_with_runs
      .into_iter()
      .fold(HashMap::new(), |mut acc, (signup, run)| {
        if let Some(run) = run {
          let event_signups = acc.entry(run.event_id).or_default();
          event_signups.push(signup);
        }

        acc
      }),
  )
}

pub async fn load_all_team_member_event_ids_in_convention(
  db: &ConnectionWrapper,
  convention_id: i64,
  user_id: Option<i64>,
) -> Result<HashSet<i64>, DbErr> {
  let events = events::Entity::find()
    .join(JoinType::InnerJoin, events::Relation::TeamMembers.def())
    .join(
      JoinType::InnerJoin,
      team_members::Relation::UserConProfiles.def(),
    )
    .filter(events::Column::ConventionId.eq(convention_id))
    .filter(user_con_profiles::Column::UserId.eq(user_id))
    .all(db)
    .await?;

  Ok(events.iter().map(|event| event.id).collect())
}
