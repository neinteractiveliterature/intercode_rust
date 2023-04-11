use async_trait::async_trait;
use intercode_entities::{model_ext::form_item_permissions::FormItemRole, user_con_profiles};
use sea_orm::DbErr;

use crate::{AuthorizationInfo, FormResponsePolicy, Policy, ReadManageAction};

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
      UserConProfileAction::Read => todo!(),
      UserConProfileAction::ReadEmail => todo!(),
      UserConProfileAction::ReadBirthDate => todo!(),
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

#[async_trait]
impl FormResponsePolicy<AuthorizationInfo, user_con_profiles::Model> for UserConProfilePolicy {
  async fn form_item_viewer_role(
    principal: &AuthorizationInfo,
    user_con_profile: &user_con_profiles::Model,
  ) -> FormItemRole {
    if principal
      .has_convention_permission("read_user_con_profiles", user_con_profile.convention_id)
      .await
      .unwrap_or(false)
    {
      if principal
        .has_convention_permission(
          "read_user_con_profile_birth_date",
          user_con_profile.convention_id,
        )
        .await
        .unwrap_or(false)
        && principal
          .has_convention_permission(
            "read_user_con_profile_email",
            user_con_profile.convention_id,
          )
          .await
          .unwrap_or(false)
        && principal
          .has_convention_permission(
            "read_user_con_profile_personal_info",
            user_con_profile.convention_id,
          )
          .await
          .unwrap_or(false)
      {
        return FormItemRole::Admin;
      } else {
        return FormItemRole::AllProfilesBasicAccess;
      }
    }

    return FormItemRole::Normal;
  }

  async fn form_item_writer_role(
    principal: &AuthorizationInfo,
    user_con_profile: &user_con_profiles::Model,
  ) -> FormItemRole {
    if principal
      .has_convention_permission("update_user_con_profiles", user_con_profile.convention_id)
      .await
      .unwrap_or(false)
    {
      if principal
        .has_convention_permission(
          "read_user_con_profile_birth_date",
          user_con_profile.convention_id,
        )
        .await
        .unwrap_or(false)
        && principal
          .has_convention_permission(
            "read_user_con_profile_email",
            user_con_profile.convention_id,
          )
          .await
          .unwrap_or(false)
        && principal
          .has_convention_permission(
            "read_user_con_profile_personal_info",
            user_con_profile.convention_id,
          )
          .await
          .unwrap_or(false)
      {
        return FormItemRole::Admin;
      } else {
        return FormItemRole::AllProfilesBasicAccess;
      }
    }

    return FormItemRole::Normal;
  }
}
