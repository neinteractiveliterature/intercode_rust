use async_trait::async_trait;
use futures::future::try_join_all;
use intercode_entities::{conventions, orders, tickets, user_con_profiles};
use sea_orm::{sea_query::Cond, ColumnTrait, DbErr, EntityTrait, QueryFilter, QuerySelect};

use crate::{AuthorizationInfo, EntityPolicy, Policy, ReadManageAction};

use super::{TicketAction, TicketPolicy};

pub enum OrderAction {
  Read,
  Manage,
  ManageCoupons,
  Submit,
  Cancel,
}

impl From<ReadManageAction> for OrderAction {
  fn from(action: ReadManageAction) -> Self {
    match action {
      ReadManageAction::Read => Self::Read,
      ReadManageAction::Manage => Self::Manage,
    }
  }
}

pub struct OrderPolicy;

#[async_trait]
impl
  Policy<
    AuthorizationInfo,
    (
      conventions::Model,
      user_con_profiles::Model,
      orders::Model,
      Vec<tickets::Model>,
    ),
  > for OrderPolicy
{
  type Action = OrderAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &OrderAction,
    (convention, user_con_profile, order, tickets): &(
      conventions::Model,
      user_con_profiles::Model,
      orders::Model,
      Vec<tickets::Model>,
    ),
  ) -> Result<bool, Self::Error> {
    if !principal.can_act_in_convention(convention.id) {
      return Ok(false);
    }

    match action {
      OrderAction::Read => Ok(
        (principal.has_scope("read_conventions")
          && (principal
            .has_convention_permission("read_orders", convention.id)
            .await?
            || try_join_all(tickets.iter().map(|ticket| async {
              TicketPolicy::action_permitted(
                principal,
                &TicketAction::Read,
                &(convention.clone(), user_con_profile.clone(), ticket.clone()),
              )
              .await
            }))
            .await?
            .into_iter()
            .any(|can_read_ticket| can_read_ticket)))
          || (principal.has_scope("read_profile")
            && principal
              .user_con_profile_ids()
              .await?
              .contains(&user_con_profile.id))
          || principal.site_admin_read(),
      ),
      OrderAction::Manage | OrderAction::Cancel => Ok(
        principal
          .has_scope_and_convention_permission("manage_conventions", "update_orders", convention.id)
          .await?
          || principal.site_admin_manage(),
      ),
      OrderAction::Submit | OrderAction::ManageCoupons => Ok(
        (principal.has_scope("manage_profile")
          && (order.status == "pending" || order.status == "unpaid")
          && principal
            .user_con_profile_ids()
            .await?
            .contains(&user_con_profile.id))
          || OrderPolicy::action_permitted(
            principal,
            &OrderAction::Manage,
            &(
              convention.clone(),
              user_con_profile.clone(),
              order.clone(),
              tickets.clone(),
            ),
          )
          .await?,
      ),
    }
  }
}

impl EntityPolicy<AuthorizationInfo, orders::Model> for OrderPolicy {
  type Action = OrderAction;

  fn id_column() -> orders::Column {
    orders::Column::Id
  }

  fn accessible_to(
    principal: &AuthorizationInfo,
    _action: &Self::Action,
  ) -> sea_orm::Select<<orders::Model as sea_orm::ModelTrait>::Entity> {
    let scope = orders::Entity::find()
      .inner_join(user_con_profiles::Entity)
      .filter(
        Cond::any()
          .add_option(if principal.has_scope("read_conventions") {
            Some(
              user_con_profiles::Column::ConventionId.in_subquery(
                QuerySelect::query(
                  &mut principal
                    .conventions_with_permission("read_orders")
                    .select_only()
                    .column(conventions::Column::Id),
                )
                .take(),
              ),
            )
          } else {
            None
          })
          .add_option(if principal.has_scope("read_conventions") {
            Some(
              user_con_profiles::Column::ConventionId.in_subquery(
                QuerySelect::query(
                  &mut principal
                    .conventions_with_permission("update_orders")
                    .select_only()
                    .column(conventions::Column::Id),
                )
                .take(),
              ),
            )
          } else {
            None
          })
          .add_option(if principal.has_scope("read_profile") {
            principal
              .user
              .as_ref()
              .map(|user| user_con_profiles::Column::UserId.eq(user.id))
          } else {
            None
          }),
      );

    if let Some(assumed_identity_from_profile) = principal.assumed_identity_from_profile.as_ref() {
      scope.filter(
        user_con_profiles::Column::ConventionId.eq(assumed_identity_from_profile.convention_id),
      )
    } else {
      scope
    }
  }
}
