use async_trait::async_trait;
use futures::future::try_join_all;
use intercode_entities::{conventions, orders, tickets, user_con_profiles};
use sea_orm::DbErr;

use crate::{AuthorizationInfo, Policy, ReadManageAction};

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
