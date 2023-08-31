use async_graphql::async_trait::async_trait;
use intercode_entities::{conventions, events, runs, signup_requests, user_con_profiles};
use intercode_policies::{AuthorizationInfo, EntityPolicy, Policy, ReadManageAction};
use sea_orm::{
  sea_query::{Cond, Expr},
  ColumnTrait, DbErr, EntityTrait, QueryFilter, QuerySelect, Select,
};

pub enum SignupRequestAction {
  Read,
  Manage,
  Accept,
  Reject,
  Create,
  Withdraw,
}

impl From<ReadManageAction> for SignupRequestAction {
  fn from(action: ReadManageAction) -> Self {
    match action {
      ReadManageAction::Read => Self::Read,
      ReadManageAction::Manage => Self::Manage,
    }
  }
}

pub struct SignupRequestPolicy;

#[async_trait]
impl
  Policy<
    AuthorizationInfo,
    (
      conventions::Model,
      events::Model,
      runs::Model,
      signup_requests::Model,
    ),
  > for SignupRequestPolicy
{
  type Action = SignupRequestAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &Self::Action,
    (convention, _event, _run, signup_request): &(
      conventions::Model,
      events::Model,
      runs::Model,
      signup_requests::Model,
    ),
  ) -> Result<bool, Self::Error> {
    if !principal.can_act_in_convention(convention.id) {
      return Ok(false);
    }

    match action {
      SignupRequestAction::Read => Ok(
        (principal.has_scope("read_signups")
          && principal
            .user_con_profile_ids()
            .await?
            .contains(&signup_request.user_con_profile_id))
          || (principal.has_scope("read_conventions")
            && convention.signup_mode == "moderated"
            && principal
              .has_convention_permission("update_signups", convention.id)
              .await?)
          || principal.site_admin_read(),
      ),
      SignupRequestAction::Manage | SignupRequestAction::Reject | SignupRequestAction::Accept => {
        Ok(
          (principal.has_scope("manage_conventions")
            && convention.signup_mode == "moderated"
            && principal
              .has_convention_permission("update_signups", convention.id)
              .await?)
            || principal.site_admin_manage(),
        )
      }
      SignupRequestAction::Create => Ok(
        principal.has_scope("manage_signups")
          && convention.signup_mode == "moderated"
          && principal
            .user_con_profile_ids()
            .await?
            .contains(&signup_request.user_con_profile_id),
      ),
      SignupRequestAction::Withdraw => Ok(
        principal.has_scope("manage_signups")
          && signup_request.state == "pending"
          && principal
            .user_con_profile_ids()
            .await?
            .contains(&signup_request.user_con_profile_id),
      ),
    }
  }
}

impl EntityPolicy<AuthorizationInfo, signup_requests::Model> for SignupRequestPolicy {
  type Action = SignupRequestAction;

  fn id_column() -> signup_requests::Column {
    signup_requests::Column::Id
  }

  fn accessible_to(
    principal: &AuthorizationInfo,
    action: &Self::Action,
  ) -> Select<signup_requests::Entity> {
    match action {
      SignupRequestAction::Read => {
        let scope = signup_requests::Entity::find();
        if principal.has_scope("update_signups") && principal.site_admin() {
          return scope;
        }

        let scope = scope.filter(
          Cond::any()
            .add_option(principal.user.as_ref().and_then(|user| {
              if principal.has_scope("read_signups") {
                Some(
                  signup_requests::Column::UserConProfileId.in_subquery(
                    sea_orm::QuerySelect::query(
                      &mut user_con_profiles::Entity::find()
                        .filter(user_con_profiles::Column::UserId.eq(user.id))
                        .select_only()
                        .column(user_con_profiles::Column::Id),
                    )
                    .take(),
                  ),
                )
              } else {
                None
              }
            }))
            .add_option(if principal.has_scope("read_conventions") {
              Some(
                signup_requests::Column::TargetRunId.in_subquery(
                  sea_orm::QuerySelect::query(
                    &mut runs::Entity::find()
                      .inner_join(events::Entity)
                      .filter(
                        events::Column::ConventionId.in_subquery(
                          sea_orm::QuerySelect::query(
                            &mut principal
                              .conventions_with_permission("update_signups")
                              .filter(conventions::Column::SignupMode.eq("moderated"))
                              .select_only()
                              .column(conventions::Column::Id),
                          )
                          .take(),
                        ),
                      )
                      .select_only()
                      .column(runs::Column::Id),
                  )
                  .take(),
                ),
              )
            } else {
              None
            }),
        );

        if let Some(assumed_identity_from_profile) = &principal.assumed_identity_from_profile {
          scope.filter(
            signup_requests::Column::TargetRunId.in_subquery(
              sea_orm::QuerySelect::query(
                &mut runs::Entity::find().inner_join(events::Entity).filter(
                  events::Column::ConventionId.eq(assumed_identity_from_profile.convention_id),
                ),
              )
              .take(),
            ),
          )
        } else {
          scope
        }
      }
      _ => signup_requests::Entity::find().filter(Expr::cust("1 = 0")),
    }
  }
}
