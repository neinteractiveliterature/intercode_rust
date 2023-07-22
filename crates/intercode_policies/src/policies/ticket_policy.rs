use std::sync::Arc;

use async_graphql::{Context, Error};
use async_trait::async_trait;
use intercode_entities::{conventions, tickets, user_con_profiles};
use intercode_graphql_loaders::LoaderManager;
use sea_orm::DbErr;
use seawater::loaders::ExpectModel;

use crate::{AuthorizationInfo, GuardablePolicy, Policy, PolicyGuard, ReadManageAction};

pub enum TicketAction {
  Read,
  Manage,
  Provide,
}

impl From<ReadManageAction> for TicketAction {
  fn from(action: ReadManageAction) -> Self {
    match action {
      ReadManageAction::Read => Self::Read,
      ReadManageAction::Manage => Self::Manage,
    }
  }
}

pub struct TicketPolicy;

#[async_trait]
impl Policy<AuthorizationInfo, (conventions::Model, user_con_profiles::Model, tickets::Model)>
  for TicketPolicy
{
  type Action = TicketAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &TicketAction,
    (convention, user_con_profile, _ticket): &(
      conventions::Model,
      user_con_profiles::Model,
      tickets::Model,
    ),
  ) -> Result<bool, Self::Error> {
    if !principal.can_act_in_convention(convention.id) {
      return Ok(false);
    }

    match action {
      TicketAction::Read => Ok(
        (principal
          .has_scope_and_convention_permission("read_conventions", "read_tickets", convention.id)
          .await?)
          || (principal.has_scope("read_events")
            && principal.team_member_in_convention(convention.id).await?)
          || {
            principal.has_scope("read_profile")
              && principal
                .user
                .as_ref()
                .map(|u| u.id == user_con_profile.id)
                .unwrap_or(false)
          }
          || principal.site_admin_read(),
      ),
      TicketAction::Manage => {
        principal
          .has_scope_and_convention_permission(
            "manage_conventions",
            "update_tickets",
            convention.id,
          )
          .await
      }
      TicketAction::Provide => todo!(),
    }
  }
}

pub struct TicketGuard {
  action: TicketAction,
  model: tickets::Model,
}

#[async_trait]
impl
  PolicyGuard<
    '_,
    TicketPolicy,
    (conventions::Model, user_con_profiles::Model, tickets::Model),
    tickets::Model,
  > for TicketGuard
{
  fn new(action: TicketAction, model: &tickets::Model) -> Self
  where
    Self: Sized,
  {
    TicketGuard {
      action,
      model: model.clone(),
    }
  }

  fn get_action(&self) -> &TicketAction {
    &self.action
  }

  fn get_model(&self) -> &tickets::Model {
    &self.model
  }

  async fn get_resource(
    &self,
    model: &tickets::Model,
    ctx: &Context<'_>,
  ) -> Result<(conventions::Model, user_con_profiles::Model, tickets::Model), Error> {
    let loaders = ctx.data::<Arc<LoaderManager>>()?;
    let ticket_user_con_profile_loader = loaders.ticket_user_con_profile();
    let user_con_profile_convention_loader = loaders.user_con_profile_convention();

    let user_con_profile_result = ticket_user_con_profile_loader.load_one(model.id).await?;
    let user_con_profile = user_con_profile_result.expect_one()?;
    let convention_result = user_con_profile_convention_loader
      .load_one(user_con_profile.id)
      .await?;
    let convention = convention_result.expect_one()?;

    Ok((convention.clone(), user_con_profile.clone(), model.clone()))
  }
}

impl
  GuardablePolicy<
    '_,
    (conventions::Model, user_con_profiles::Model, tickets::Model),
    tickets::Model,
  > for TicketPolicy
{
  type Guard = TicketGuard;
}
