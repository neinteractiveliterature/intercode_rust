use std::collections::{HashMap, HashSet};

use intercode_entities::{
  cms_content_groups, event_categories, organization_roles_users, permissions, staff_positions,
  staff_positions_user_con_profiles, user_con_profiles,
};
use sea_orm::{
  sea_query::{Alias, Expr},
  ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, FromQueryResult, JoinType,
  QueryFilter, QuerySelect, RelationTrait, Select,
};

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
                  staff_positions::Relation::StaffPositionsUserConProfiles.def(),
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

pub struct UserPermission {
  pub permission: String,
  pub model_id: PermissionModelId,
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
    })
  }
}

#[derive(Debug, Clone, Default)]
pub struct UserPermissionsMap {
  permissions: HashMap<PermissionModelId, HashSet<String>>,
}

impl UserPermissionsMap {
  pub fn from_user_permissions(iter: &mut dyn Iterator<Item = UserPermission>) -> Self {
    let permissions = iter
      .map(|user_permission| (user_permission.model_id, user_permission.permission))
      .fold(HashMap::new(), |mut acc, (model_id, permission)| {
        let entry = acc.entry(model_id);
        let permissions: &mut HashSet<String> = entry.or_default();
        permissions.insert(permission);
        acc
      });

    Self { permissions }
  }

  pub fn has_permission(&self, model_id: &PermissionModelId, permission: &str) -> bool {
    self
      .permissions
      .get(model_id)
      .map(|perm_set| perm_set.contains(permission))
      .unwrap_or(false)
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
}

pub async fn load_all_permissions_in_convention_with_model_type_and_id(
  db: &DatabaseConnection,
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
