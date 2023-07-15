use std::marker::PhantomData;

use async_trait::async_trait;
use intercode_entities::{
  cms_content_group_associations, cms_content_groups, cms_content_model::CmsContentModel,
  cms_files, cms_graphql_queries, cms_layouts, cms_partials, cms_variables, conventions, pages,
  permissions, root_sites,
};
use intercode_policies::{
  user_permission_scope, AuthorizationInfo, EntityPolicy, Policy, ReadManageAction,
};
use sea_orm::{
  sea_query::{Expr, UnionType},
  ColumnTrait, DbErr, EntityTrait, ModelTrait, QueryFilter, QuerySelect, QueryTrait, Select,
};

pub trait CmsContentAuthorizable: Send + Sync + ModelTrait {
  fn convention_id(&self) -> Option<i64>;
  fn cms_content_groups_scope(&self) -> Option<Select<cms_content_groups::Entity>>;
}

macro_rules! cms_content_model_authorizable {
  ($model: path) => {
    impl CmsContentAuthorizable for $model {
      fn convention_id(&self) -> Option<i64> {
        CmsContentModel::convention_id(self)
      }

      fn cms_content_groups_scope(&self) -> Option<Select<cms_content_groups::Entity>> {
        Some(CmsContentModel::cms_content_groups_scope(self))
      }
    }
  };
}

cms_content_model_authorizable!(pages::Model);
cms_content_model_authorizable!(cms_content_groups::Model);
cms_content_model_authorizable!(cms_graphql_queries::Model);
cms_content_model_authorizable!(cms_layouts::Model);
cms_content_model_authorizable!(cms_partials::Model);
cms_content_model_authorizable!(cms_files::Model);
cms_content_model_authorizable!(cms_variables::Model);

impl CmsContentAuthorizable for conventions::Model {
  fn convention_id(&self) -> Option<i64> {
    Some(self.id)
  }

  fn cms_content_groups_scope(&self) -> Option<Select<cms_content_groups::Entity>> {
    None
  }
}

impl CmsContentAuthorizable for root_sites::Model {
  fn convention_id(&self) -> Option<i64> {
    None
  }

  fn cms_content_groups_scope(&self) -> Option<Select<cms_content_groups::Entity>> {
    None
  }
}

pub struct CmsContentPolicy<M: CmsContentAuthorizable> {
  _phantom: PhantomData<M>,
}

async fn can_manage_all_cms_content_in_convention<M: CmsContentAuthorizable>(
  principal: &AuthorizationInfo,
  resource: &M,
) -> Result<bool, DbErr> {
  if let Some(convention_id) = resource.convention_id() {
    if !principal.can_act_in_convention(convention_id) {
      return Ok(false);
    }

    Ok(
      principal.has_scope("manage_conventions")
        && (principal.has_convention_permission("update_cms_content", convention_id)).await?,
    )
  } else {
    Ok(false)
  }
}

async fn can_manage_cms_content_via_group<M: CmsContentAuthorizable>(
  principal: &AuthorizationInfo,
  resource: &M,
) -> Result<bool, DbErr> {
  let Some(convention_id) = resource.convention_id() else {
    return Ok(false);
  };

  if !principal.can_act_in_convention(convention_id) {
    return Ok(false);
  }

  let Some(scope) = resource.cms_content_groups_scope() else { return Ok(false);};

  Ok(
    principal.has_scope("manage_conventions")
      && principal
        .cms_content_group_scope_has_permission(scope, convention_id, "update_content")
        .await?,
  )
}

async fn can_manage_because_admin<M: CmsContentAuthorizable>(
  principal: &AuthorizationInfo,
  _resource: &M,
) -> Result<bool, DbErr> {
  Ok(principal.site_admin_manage())
}

#[async_trait]
impl<M: CmsContentAuthorizable> Policy<AuthorizationInfo, M> for CmsContentPolicy<M> {
  type Action = ReadManageAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &Self::Action,
    resource: &M,
  ) -> Result<bool, Self::Error> {
    match action {
      ReadManageAction::Read => Ok(true),
      ReadManageAction::Manage => Ok(
        can_manage_all_cms_content_in_convention(principal, resource).await?
          || can_manage_because_admin(principal, resource).await?
          || can_manage_cms_content_via_group(principal, resource).await?,
      ),
    }
  }
}

impl<M: CmsContentAuthorizable> EntityPolicy<AuthorizationInfo, M> for CmsContentPolicy<M>
where
  M: CmsContentModel + ModelTrait,
{
  type Action = ReadManageAction;

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

  fn id_column() -> <<M as ModelTrait>::Entity as EntityTrait>::Column {
    <M as CmsContentModel>::id_column()
  }
}
