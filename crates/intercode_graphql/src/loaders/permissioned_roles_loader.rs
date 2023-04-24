use std::{collections::HashMap, sync::Arc};

use async_graphql::dataloader::{DataLoader, Loader};
use async_trait::async_trait;
use futures::try_join;
use intercode_entities::{
  model_ext::permissions::PermissionedRoleRef, organization_roles, permissions, staff_positions,
};
use sea_orm::DbErr;
use seawater::{
  loaders::{EntityRelationLoader, ExpectModel},
  ConnectionWrapper,
};
use std::time::Duration;

use super::exclusive_arc_utils::merge_hash_maps;
use crate::exclusive_arc_variant_loader;

pub struct PermissionedRolesLoader {
  permission_organization_role_loader:
    DataLoader<EntityRelationLoader<permissions::Entity, organization_roles::Entity>>,
  permission_staff_position_loader:
    DataLoader<EntityRelationLoader<permissions::Entity, staff_positions::Entity>>,
}

impl PermissionedRolesLoader {
  pub fn new(db: ConnectionWrapper, delay: Duration) -> Self {
    Self {
      permission_organization_role_loader: DataLoader::new(
        EntityRelationLoader::new(db.clone(), permissions::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay),
      permission_staff_position_loader: DataLoader::new(
        EntityRelationLoader::new(db, permissions::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay),
    }
  }
}

#[derive(Clone)]
pub enum PermissionedRole {
  OrganizationRole(organization_roles::Model),
  StaffPosition(staff_positions::Model),
}

exclusive_arc_variant_loader!(
  load_organization_roles,
  organization_roles::Entity,
  PermissionedRoleRef,
  PermissionedRoleRef::OrganizationRole,
  PermissionedRole,
  PermissionedRole::OrganizationRole
);

exclusive_arc_variant_loader!(
  load_staff_positions,
  staff_positions::Entity,
  PermissionedRoleRef,
  PermissionedRoleRef::StaffPosition,
  PermissionedRole,
  PermissionedRole::StaffPosition
);

#[async_trait]
impl Loader<PermissionedRoleRef> for PermissionedRolesLoader {
  type Value = PermissionedRole;
  type Error = Arc<DbErr>;

  async fn load(
    &self,
    keys: &[PermissionedRoleRef],
  ) -> Result<HashMap<PermissionedRoleRef, Self::Value>, Self::Error> {
    let (organization_roles, staff_positions) = try_join!(
      load_organization_roles(keys, &self.permission_organization_role_loader),
      load_staff_positions(keys, &self.permission_staff_position_loader),
    )?;

    Ok(merge_hash_maps(vec![organization_roles, staff_positions]))
  }
}

impl ExpectModel<PermissionedRole> for Option<PermissionedRole> {
  fn expect_model(&self) -> Result<PermissionedRole, async_graphql::Error> {
    if let Some(model) = self {
      Ok(model.to_owned())
    } else {
      Err(async_graphql::Error::new("Permissioned role not found"))
    }
  }
}
