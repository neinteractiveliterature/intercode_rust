use async_trait::async_trait;
use intercode_entities::{conventions, team_members, user_con_profiles};
use sea_orm::{
  sea_query::{Cond, Expr},
  ColumnTrait, DbErr, EntityTrait, QueryFilter, QuerySelect,
};

use crate::{AuthorizationInfo, EntityPolicy, Policy, ReadManageAction, SimpleGuardablePolicy};

use super::TeamMemberPolicy;

#[derive(PartialEq, Eq)]
pub enum UserConProfileAction {
  Read,
  ReadEmail,
  ReadBirthDate,
  ReadPersonalInfo,
  Create,
  Update,
  Delete,
  Become,
  WithdrawAllSignups,
}

impl From<ReadManageAction> for UserConProfileAction {
  fn from(action: ReadManageAction) -> Self {
    match action {
      ReadManageAction::Read => Self::Read,
      ReadManageAction::Manage => Self::Update,
    }
  }
}

fn profile_is_user_or_identity_assumer(
  principal: &AuthorizationInfo,
  user_con_profile: &user_con_profiles::Model,
) -> bool {
  if let Some(user) = &principal.user {
    if user.id == user_con_profile.user_id {
      return true;
    }
  }

  if let Some(assumed_identity_from_profile) = &principal.assumed_identity_from_profile {
    if assumed_identity_from_profile.convention_id == user_con_profile.convention_id {
      return true;
    }
  }

  false
}

pub struct UserConProfilePolicy;

#[async_trait]
impl Policy<AuthorizationInfo, user_con_profiles::Model> for UserConProfilePolicy {
  type Action = UserConProfileAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &Self::Action,
    user_con_profile: &user_con_profiles::Model,
  ) -> Result<bool, Self::Error> {
    if !principal.can_act_in_convention(user_con_profile.convention_id) {
      return Ok(false);
    }

    match action {
      UserConProfileAction::Read => Ok(
        UserConProfilePolicy::action_permitted(
          principal,
          &UserConProfileAction::ReadEmail,
          user_con_profile,
        )
        .await?
          || principal
            .is_user_con_profile_bio_eligible(user_con_profile.id, user_con_profile.convention_id)
            .await?
          || (principal.has_scope("read_events")
            && principal
              .user_con_profile_ids_in_signed_up_runs()
              .await?
              .contains(&user_con_profile.id))
          || principal
            .has_scope_and_convention_permission(
              "read_conventions",
              "read_user_con_profiles",
              user_con_profile.convention_id,
            )
            .await?,
      ),
      UserConProfileAction::ReadEmail => Ok(
        UserConProfilePolicy::action_permitted(
          principal,
          &UserConProfileAction::ReadPersonalInfo,
          user_con_profile,
        )
        .await?
          || principal
            .has_scope_and_convention_permission(
              "read_conventions",
              "read_user_con_profile_email",
              user_con_profile.convention_id,
            )
            .await?,
      ),
      UserConProfileAction::ReadBirthDate => Ok(
        (principal.has_scope("read_profile")
          && profile_is_user_or_identity_assumer(principal, user_con_profile))
          || (principal.has_scope("read_conventions")
            && (principal.has_convention_permission(
              "read_user_con_profile_birth_date",
              user_con_profile.convention_id,
            ))
            .await?)
          || principal.site_admin_read(),
      ),
      UserConProfileAction::ReadPersonalInfo => Ok(
        (principal.has_scope("read_profile")
          && profile_is_user_or_identity_assumer(principal, user_con_profile))
          || (principal.has_scope("read_conventions")
            && (principal
              .has_convention_permission(
                "read_user_con_profile_personal_info",
                user_con_profile.convention_id,
              )
              .await?
              || principal
                .has_event_category_permission_in_convention(
                  "read_event_proposals",
                  user_con_profile.convention_id,
                )
                .await?
              || principal
                .team_member_in_convention(user_con_profile.convention_id)
                .await?))
          || principal.site_admin_read(),
      ),
      UserConProfileAction::Update | UserConProfileAction::Create => {
        if !principal.can_act_in_convention(user_con_profile.convention_id) {
          return Ok(false);
        }

        if principal.has_scope("manage_profile")
          && principal.user.as_ref().map(|u| u.id) == Some(user_con_profile.id)
        {
          return Ok(true);
        }

        UserConProfilePolicy::action_permitted(
          principal,
          &UserConProfileAction::Delete,
          user_con_profile,
        )
        .await
      }
      UserConProfileAction::Delete
      | UserConProfileAction::WithdrawAllSignups
      | UserConProfileAction::Become => {
        if !principal.can_act_in_convention(user_con_profile.convention_id) {
          return Ok(false);
        }

        if principal
          .has_scope_and_convention_permission(
            "manage_conventions",
            "update_user_con_profiles",
            user_con_profile.convention_id,
          )
          .await?
        {
          return Ok(true);
        }

        Ok(principal.site_admin_manage())
      }
    }
  }
}

impl EntityPolicy<AuthorizationInfo, user_con_profiles::Model> for UserConProfilePolicy {
  type Action = UserConProfileAction;

  fn accessible_to(
    principal: &AuthorizationInfo,
    action: &Self::Action,
  ) -> sea_orm::Select<user_con_profiles::Entity> {
    let scope = user_con_profiles::Entity::find();

    // TODO consider implementing other actions
    if *action != UserConProfileAction::Read {
      return scope.filter(Expr::cust("1 = 0"));
    }

    let scope = principal
      .assumed_identity_from_profile
      .as_ref()
      .map(|profile| {
        scope
          .clone()
          .filter(user_con_profiles::Column::ConventionId.eq(profile.convention_id))
      })
      .unwrap_or(scope);

    if principal.site_admin_read() && principal.has_scope("read_conventions") {
      return scope;
    }

    scope.filter(
      Cond::any()
        .add_option(
          principal
            .user
            .as_ref()
            .map(|user| user_con_profiles::Column::UserId.eq(user.id)),
        )
        .add(
          user_con_profiles::Column::Id.in_subquery(
            QuerySelect::query(
              &mut TeamMemberPolicy::accessible_to(principal, &ReadManageAction::Read)
                .select_only()
                .column(team_members::Column::UserConProfileId),
            )
            .take(),
          ),
        )
        .add(
          user_con_profiles::Column::ConventionId.in_subquery(
            QuerySelect::query(
              &mut principal
                .conventions_where_team_member()
                .select_only()
                .column(conventions::Column::Id),
            )
            .take(),
          ),
        )
        .add(
          user_con_profiles::Column::ConventionId.in_subquery(
            QuerySelect::query(
              &mut principal
                .conventions_with_permissions(&[
                  "read_user_con_profiles",
                  "read_user_con_profile_email",
                  "read_user_con_profile_personal_info",
                ])
                .select_only()
                .column(conventions::Column::Id),
            )
            .take(),
          ),
        ),
    )
  }

  fn id_column() -> user_con_profiles::Column {
    user_con_profiles::Column::Id
  }
}

impl SimpleGuardablePolicy<'_, user_con_profiles::Model> for UserConProfilePolicy {}
