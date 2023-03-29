use std::marker::PhantomData;

use async_trait::async_trait;
use intercode_entities::{
  cms_content_group_associations, cms_content_groups, cms_files, conventions, root_sites,
};
use sea_orm::{sea_query::Expr, ColumnTrait, DbErr, EntityTrait, QueryFilter, Select};

fn cms_content_groups_scope_for_convention_id(
  convention_id: Option<i64>,
) -> Select<cms_content_groups::Entity> {
  if let Some(convention_id) = convention_id {
    cms_content_groups::Entity::find()
      .filter(cms_content_groups::Column::ParentId.eq(convention_id))
      .filter(cms_content_groups::Column::ParentType.eq("Convention"))
  } else {
    cms_content_groups::Entity::find()
      .filter(cms_content_groups::Column::ParentType.is_null())
      .filter(cms_content_groups::Column::ParentId.is_null())
  }
}

pub trait CmsContentModel: sea_orm::ModelTrait + Sync {
  fn convention_id(&self) -> Option<i64>;
  fn cms_content_groups_scope(&self) -> Select<cms_content_groups::Entity>;
}

impl CmsContentModel for cms_files::Model {
  fn convention_id(&self) -> Option<i64> {
    if !matches!(self.parent_type.as_deref(), Some("Convention")) {
      return None;
    }

    self.parent_id
  }

  fn cms_content_groups_scope(&self) -> Select<cms_content_groups::Entity> {
    let scope = cms_content_groups_scope_for_convention_id(self.convention_id());

    scope
      .inner_join(cms_content_group_associations::Entity)
      .filter(cms_content_group_associations::Column::ContentType.eq("CmsFile"))
      .filter(cms_content_group_associations::Column::ContentId.eq(self.id))
  }
}

impl CmsContentModel for conventions::Model {
  fn convention_id(&self) -> Option<i64> {
    Some(self.id)
  }

  fn cms_content_groups_scope(&self) -> Select<cms_content_groups::Entity> {
    // This is asking about general permissions across a convention, groups don't apply here
    cms_content_groups::Entity::find().filter(Expr::cust("1 = 0"))
  }
}

impl CmsContentModel for root_sites::Model {
  fn convention_id(&self) -> Option<i64> {
    None
  }

  fn cms_content_groups_scope(&self) -> Select<cms_content_groups::Entity> {
    // This is asking about general permissions across the root site, groups don't apply here
    cms_content_groups::Entity::find().filter(Expr::cust("1 = 0"))
  }
}

use crate::{AuthorizationInfo, Policy, ReadManageAction};

pub struct CmsContentPolicy<M: CmsContentModel> {
  _phantom: PhantomData<M>,
}

#[async_trait]
impl<M: CmsContentModel> Policy<AuthorizationInfo, M> for CmsContentPolicy<M> {
  type Action = ReadManageAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &Self::Action,
    resource: &M,
  ) -> Result<bool, Self::Error> {
    match action {
      ReadManageAction::Read => Ok(true),
      ReadManageAction::Manage => {
        if let Some(convention_id) = resource.convention_id() {
          if !principal.can_act_in_convention(convention_id) {
            return Ok(false);
          }

          if principal.has_scope("manage_conventions")
            && ((principal.has_convention_permission("update_cms_content", convention_id)).await?
              || (principal
                .cms_content_group_scope_has_permission(
                  resource.cms_content_groups_scope(),
                  convention_id,
                  "update_content",
                )
                .await?))
          {
            return Ok(true);
          }
        }

        return Ok(principal.site_admin_manage());
      }
    }
  }
}
