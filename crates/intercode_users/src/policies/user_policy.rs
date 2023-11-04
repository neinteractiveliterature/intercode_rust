use axum::async_trait;
use intercode_entities::{conventions, user_con_profiles, users};
use intercode_policies::{
  conventions_with_organization_permission, AuthorizationInfo, EntityPolicy, Policy,
  ReadManageAction,
};
use sea_orm::{
  sea_query::Cond, ColumnTrait, DbErr, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter,
  QuerySelect, QueryTrait,
};

pub enum UserAction {
  Read,
  Manage,
  Update,
  Merge,
}

impl From<ReadManageAction> for UserAction {
  fn from(value: ReadManageAction) -> Self {
    match value {
      ReadManageAction::Read => UserAction::Read,
      ReadManageAction::Manage => UserAction::Manage,
    }
  }
}

pub struct UserPolicy;

#[async_trait]
impl Policy<AuthorizationInfo, users::Model> for UserPolicy {
  type Action = UserAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &UserAction,
    resource: &users::Model,
  ) -> Result<bool, Self::Error> {
    match action {
      UserAction::Read => {
        if principal
          .user
          .as_ref()
          .is_some_and(|user| user.id == resource.id)
        {
          return Ok(true);
        }

        if let Some(profile) = principal.assumed_identity_from_profile.as_ref() {
          if resource
            .find_related(user_con_profiles::Entity)
            .filter(user_con_profiles::Column::ConventionId.eq(profile.convention_id))
            .count(&principal.db)
            .await?
            == 0
          {
            return Ok(false);
          }
        }

        Ok(
          principal.has_scope("read_organizations")
            && (principal.site_admin_read()
              || principal
                .conventions_with_organization_permission("read_convention_users")
                .count(&principal.db)
                .await?
                > 0),
        )
      }
      _ => todo!(),
    }
  }
}

impl EntityPolicy<AuthorizationInfo, users::Model> for UserPolicy {
  type Action = UserAction;

  fn accessible_to(
    principal: &AuthorizationInfo,
    action: &Self::Action,
  ) -> sea_orm::Select<users::Entity> {
    match action {
      UserAction::Read => {
        let mut scope = users::Entity::find();

        if principal.site_admin() && principal.has_scope("read_organizations") {
          return scope;
        }

        if let Some(user) = principal.user.as_ref() {
          scope = scope.filter(Cond::any().add(users::Column::Id.eq(user.id)).add_option({
            if principal.has_scope("read_organizations") {
              Some(
                users::Column::Id.in_subquery(
                  user_con_profiles::Entity::find()
                    .filter(
                      user_con_profiles::Column::ConventionId.in_subquery(
                        conventions_with_organization_permission(
                          "read_convention_users",
                          Some(user.id),
                        )
                        .select_only()
                        .column(conventions::Column::Id)
                        .into_query(),
                      ),
                    )
                    .select_only()
                    .column(user_con_profiles::Column::UserId)
                    .into_query(),
                ),
              )
            } else {
              None
            }
          }));
        }

        if let Some(profile) = principal.assumed_identity_from_profile.as_ref() {
          scope = scope.filter(
            users::Column::Id.in_subquery(
              user_con_profiles::Entity::find()
                .filter(user_con_profiles::Column::Id.eq(profile.convention_id))
                .select_only()
                .column(user_con_profiles::Column::UserId)
                .into_query(),
            ),
          )
        }

        scope
      }
      _ => todo!(),
    }
  }

  fn id_column() -> users::Column {
    users::Column::Id
  }
}
