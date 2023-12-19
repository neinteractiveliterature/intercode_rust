use axum::async_trait;
use intercode_entities::{conventions, events, runs, signup_changes, user_con_profiles};
use intercode_policies::{AuthorizationInfo, EntityPolicy, Policy, ReadManageAction};
use sea_orm::{
  sea_query::{Cond, Expr},
  ColumnTrait, DbErr, EntityTrait, QueryFilter, QuerySelect, Select,
};

pub struct SignupChangePolicy;

#[async_trait]
impl
  Policy<
    AuthorizationInfo,
    (
      conventions::Model,
      events::Model,
      runs::Model,
      signup_changes::Model,
    ),
  > for SignupChangePolicy
{
  type Action = ReadManageAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &Self::Action,
    (convention, _event, _run, signup_change): &(
      conventions::Model,
      events::Model,
      runs::Model,
      signup_changes::Model,
    ),
  ) -> Result<bool, Self::Error> {
    if !principal.can_act_in_convention(convention.id) {
      return Ok(false);
    }

    match action {
      ReadManageAction::Read => Ok(
        principal
          .has_scope_and_convention_permission(
            "read_conventions",
            "read_signup_details",
            convention.id,
          )
          .await?
          || (principal.has_scope("read_signups")
            && principal
              .user_con_profile_ids()
              .await?
              .contains(&signup_change.user_con_profile_id))
          || (principal.has_scope("read_events")
            && principal
              .team_member_run_ids_in_convention(convention.id)
              .await?
              .contains(&signup_change.run_id)),
      ),
      ReadManageAction::Manage => {
        // it's impossible to change a signup change once created
        Ok(false)
      }
    }
  }
}

impl EntityPolicy<AuthorizationInfo, signup_changes::Model> for SignupChangePolicy {
  type Action = ReadManageAction;

  fn accessible_to(
    principal: &AuthorizationInfo,
    action: &Self::Action,
  ) -> Select<signup_changes::Entity> {
    match action {
      ReadManageAction::Read => {
        if principal.site_admin_read() {
          signup_changes::Entity::find()
        } else {
          signup_changes::Entity::find().filter(
            Cond::any()
              .add_option(if principal.has_scope("read_signups") {
                if let Some(user) = &principal.user {
                  Some(
                    signup_changes::Column::UserConProfileId.in_subquery(
                      QuerySelect::query(
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
              } else {
                None
              })
              .add_option({
                if principal.has_scope("read_events") {
                  Some(
                    signup_changes::Column::RunId.in_subquery(
                      QuerySelect::query(
                        &mut runs::Entity::find()
                          .filter(
                            runs::Column::EventId.in_subquery(
                              QuerySelect::query(
                                &mut principal
                                  .events_where_team_member()
                                  .select_only()
                                  .column(events::Column::Id),
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
                }
              })
              .add_option({
                if principal.has_scope("read_conventions") {
                  Some(
                    signup_changes::Column::RunId.in_subquery(
                      QuerySelect::query(
                        &mut runs::Entity::find().filter(
                          runs::Column::EventId.in_subquery(
                            QuerySelect::query(
                              &mut events::Entity::find().filter(
                                events::Column::ConventionId.in_subquery(
                                  QuerySelect::query(
                                    &mut principal
                                      .conventions_with_permission("read_signup_details"),
                                  )
                                  .take(),
                                ),
                              ),
                            )
                            .take(),
                          ),
                        ),
                      )
                      .take(),
                    ),
                  )
                } else {
                  None
                }
              }),
          )
        }
      }
      ReadManageAction::Manage => signup_changes::Entity::find().filter(Expr::cust("1 = 0")),
    }
  }

  fn id_column() -> signup_changes::Column {
    signup_changes::Column::Id
  }
}
