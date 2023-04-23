use std::marker::PhantomData;

use async_trait::async_trait;
use intercode_entities::{
  cms_content_group_associations, cms_content_groups, cms_files, cms_parent::CmsParent,
  conventions, pages, permissions, root_sites,
};
use sea_orm::{
  sea_query::{Expr, SelectStatement, UnionType},
  ColumnTrait, DbErr, EntityTrait, ModelTrait, QueryFilter, QuerySelect, QueryTrait, Select,
};

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
  fn cms_content_group_association_model_name() -> &'static str;
  fn filter_by_id_in(
    scope: Select<<Self as ModelTrait>::Entity>,
    subquery: SelectStatement,
  ) -> Select<<Self as ModelTrait>::Entity>;
  fn filter_by_parent_id(
    scope: Select<<Self as ModelTrait>::Entity>,
    parent_type: &str,
    subquery: SelectStatement,
  ) -> Select<<Self as ModelTrait>::Entity>;
  fn id_column() -> <<Self as ModelTrait>::Entity as EntityTrait>::Column;
  fn filter_by_parent(
    scope: Select<<Self as ModelTrait>::Entity>,
    parent: &CmsParent,
  ) -> Select<<Self as ModelTrait>::Entity>;
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

  fn cms_content_group_association_model_name() -> &'static str {
    "CmsFile"
  }

  fn filter_by_id_in(
    scope: Select<<Self as ModelTrait>::Entity>,
    subquery: SelectStatement,
  ) -> Select<<Self as ModelTrait>::Entity> {
    scope.filter(cms_files::Column::Id.in_subquery(subquery))
  }

  fn filter_by_parent_id(
    scope: Select<<Self as ModelTrait>::Entity>,
    parent_type: &str,
    subquery: SelectStatement,
  ) -> Select<<Self as ModelTrait>::Entity> {
    scope
      .filter(cms_files::Column::ParentType.eq(parent_type))
      .filter(cms_files::Column::ParentId.in_subquery(subquery))
  }

  fn id_column() -> <<Self as ModelTrait>::Entity as EntityTrait>::Column {
    cms_files::Column::Id
  }

  fn filter_by_parent(
    scope: Select<<Self as ModelTrait>::Entity>,
    parent: &CmsParent,
  ) -> Select<<Self as ModelTrait>::Entity> {
    match parent {
      CmsParent::Convention(convention) => scope
        .filter(cms_files::Column::ParentType.eq("Convention"))
        .filter(cms_files::Column::ParentId.eq(convention.id)),
      CmsParent::RootSite(root_site) => scope
        .filter(cms_files::Column::ParentType.eq("RootSite"))
        .filter(cms_files::Column::ParentId.eq(root_site.id)),
    }
  }
}

impl CmsContentModel for pages::Model {
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
      .filter(cms_content_group_associations::Column::ContentType.eq("Page"))
      .filter(cms_content_group_associations::Column::ContentId.eq(self.id))
  }

  fn cms_content_group_association_model_name() -> &'static str {
    "Page"
  }

  fn filter_by_id_in(
    scope: Select<<Self as ModelTrait>::Entity>,
    subquery: SelectStatement,
  ) -> Select<<Self as ModelTrait>::Entity> {
    scope.filter(pages::Column::Id.in_subquery(subquery))
  }

  fn filter_by_parent_id(
    scope: Select<<Self as ModelTrait>::Entity>,
    parent_type: &str,
    subquery: SelectStatement,
  ) -> Select<<Self as ModelTrait>::Entity> {
    scope
      .filter(pages::Column::ParentType.eq(parent_type))
      .filter(pages::Column::ParentId.in_subquery(subquery))
  }

  fn id_column() -> <<Self as ModelTrait>::Entity as EntityTrait>::Column {
    pages::Column::Id
  }

  fn filter_by_parent(
    scope: Select<<Self as ModelTrait>::Entity>,
    parent: &CmsParent,
  ) -> Select<<Self as ModelTrait>::Entity> {
    match parent {
      CmsParent::Convention(convention) => scope
        .filter(pages::Column::ParentType.eq("Convention"))
        .filter(pages::Column::ParentId.eq(convention.id)),
      CmsParent::RootSite(root_site) => scope
        .filter(pages::Column::ParentType.eq("RootSite"))
        .filter(pages::Column::ParentId.eq(root_site.id)),
    }
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

  fn cms_content_group_association_model_name() -> &'static str {
    "Convention"
  }

  fn filter_by_id_in(
    scope: Select<<Self as ModelTrait>::Entity>,
    subquery: SelectStatement,
  ) -> Select<<Self as ModelTrait>::Entity> {
    scope.filter(conventions::Column::Id.in_subquery(subquery))
  }

  fn filter_by_parent_id(
    _scope: Select<<Self as ModelTrait>::Entity>,
    _parent_type: &str,
    _subquery: SelectStatement,
  ) -> Select<<Self as ModelTrait>::Entity> {
    // Conventions have no parent
    conventions::Entity::find().filter(Expr::cust("1 = 0"))
  }

  fn filter_by_parent(
    _scope: Select<<Self as ModelTrait>::Entity>,
    _parent: &CmsParent,
  ) -> Select<<Self as ModelTrait>::Entity> {
    // Conventions have no parent
    conventions::Entity::find().filter(Expr::cust("1 = 0"))
  }

  fn id_column() -> <<Self as ModelTrait>::Entity as EntityTrait>::Column {
    conventions::Column::Id
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

  fn cms_content_group_association_model_name() -> &'static str {
    "RootSite"
  }

  fn filter_by_id_in(
    scope: Select<<Self as ModelTrait>::Entity>,
    subquery: SelectStatement,
  ) -> Select<<Self as ModelTrait>::Entity> {
    scope.filter(root_sites::Column::Id.in_subquery(subquery))
  }

  fn filter_by_parent_id(
    _scope: Select<<Self as ModelTrait>::Entity>,
    _parent_type: &str,
    _subquery: SelectStatement,
  ) -> Select<<Self as ModelTrait>::Entity> {
    // Root sites have no parent
    root_sites::Entity::find().filter(Expr::cust("1 = 0"))
  }

  fn filter_by_parent(
    _scope: Select<<Self as ModelTrait>::Entity>,
    _parent: &CmsParent,
  ) -> Select<<Self as ModelTrait>::Entity> {
    // Root sites have no parent
    root_sites::Entity::find().filter(Expr::cust("1 = 0"))
  }

  fn id_column() -> <<Self as ModelTrait>::Entity as EntityTrait>::Column {
    root_sites::Column::Id
  }
}

use crate::{user_permission_scope, AuthorizationInfo, EntityPolicy, Policy, ReadManageAction};

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

impl<M: CmsContentModel> EntityPolicy<AuthorizationInfo, M> for CmsContentPolicy<M> {
  fn accessible_to(
    principal: &AuthorizationInfo,
    action: &ReadManageAction,
  ) -> Select<<M as sea_orm::ModelTrait>::Entity> {
    let scope = <<M as ModelTrait>::Entity as EntityTrait>::find();
    match action {
      ReadManageAction::Read => scope,
      ReadManageAction::Manage => {
        if !principal.has_scope("manage_conventions") {
          return scope.filter(Expr::cust("0 = 1"));
        }

        if principal.site_admin() {
          return scope;
        }

        let user_permissions = user_permission_scope(principal.user.as_ref().map(|u| u.id));
        let conventions_with_update_cms_content_permissions = user_permissions
          .clone()
          .filter(permissions::Column::ConventionId.is_not_null())
          .filter(permissions::Column::Permission.eq("update_cms_content"));
        let cms_content_groups_with_update_content_permissions = user_permissions
          .filter(permissions::Column::CmsContentGroupId.is_not_null())
          .filter(permissions::Column::Permission.eq("update_content"));

        let contents_with_applicable_convention = M::filter_by_parent_id(
          scope.clone(),
          "Convention",
          conventions_with_update_cms_content_permissions
            .select_only()
            .column(permissions::Column::ConventionId)
            .into_query(),
        );

        let applicable_associations =
          <cms_content_group_associations::Entity as EntityTrait>::find()
            .filter(
              cms_content_group_associations::Column::ContentType
                .eq(M::cms_content_group_association_model_name()),
            )
            .filter(
              cms_content_group_associations::Column::CmsContentGroupId.in_subquery(
                cms_content_groups_with_update_content_permissions
                  .select_only()
                  .column(permissions::Column::CmsContentGroupId)
                  .into_query(),
              ),
            );

        let mut group_content_query = applicable_associations
          .select_only()
          .column(cms_content_group_associations::Column::ContentId)
          .into_query();

        let content_ids = group_content_query.union(
          UnionType::Distinct,
          contents_with_applicable_convention
            .select_only()
            .column(M::id_column())
            .into_query(),
        );

        M::filter_by_id_in(scope, content_ids.to_owned())
      }
    }
  }
}
