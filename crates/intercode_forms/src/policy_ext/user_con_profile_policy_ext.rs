use async_trait::async_trait;
use intercode_entities::{model_ext::form_item_permissions::FormItemRole, user_con_profiles};
use intercode_policies::{policies::UserConProfilePolicy, AuthorizationInfo};

use super::FormResponsePolicy;

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
