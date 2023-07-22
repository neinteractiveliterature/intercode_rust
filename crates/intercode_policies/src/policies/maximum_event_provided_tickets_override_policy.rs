use std::sync::Arc;

use async_graphql::{Context, Error};
use axum::async_trait;
use intercode_entities::{conventions, events, maximum_event_provided_tickets_overrides};
use intercode_graphql_loaders::LoaderManager;
use sea_orm::DbErr;
use seawater::loaders::ExpectModel;

use crate::{
  authorization_info::AuthorizationInfo,
  policy::{Policy, ReadManageAction},
  GuardablePolicy, PolicyGuard,
};

use super::{EventAction, EventPolicy};

pub struct MaximumEventProvidedTicketsOverridePolicy;

#[async_trait]
impl
  Policy<
    AuthorizationInfo,
    (
      conventions::Model,
      events::Model,
      maximum_event_provided_tickets_overrides::Model,
    ),
  > for MaximumEventProvidedTicketsOverridePolicy
{
  type Action = ReadManageAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &ReadManageAction,
    (convention, event, _mepto): &(
      conventions::Model,
      events::Model,
      maximum_event_provided_tickets_overrides::Model,
    ),
  ) -> Result<bool, Self::Error> {
    match action {
      ReadManageAction::Read => Ok(
        principal.has_scope("read_events")
          && (principal
            .has_convention_permission("override_event_tickets", convention.id)
            .await?
            || principal
              .has_event_category_permission(
                "override_event_tickets",
                convention.id,
                event.event_category_id,
              )
              .await?
            || principal
              .team_member_event_ids_in_convention(convention.id)
              .await?
              .contains(&event.id))
          || principal.site_admin_read(),
      ),
      ReadManageAction::Manage => {
        EventPolicy::action_permitted(
          principal,
          &EventAction::OverrideMaximumEventProvidedTickets,
          &(convention.clone(), event.clone()),
        )
        .await
      }
    }
  }
}

pub struct MaximumEventProvidedTicketsOverrideGuard {
  action: ReadManageAction,
  model: maximum_event_provided_tickets_overrides::Model,
}

#[async_trait]
impl
  PolicyGuard<
    '_,
    MaximumEventProvidedTicketsOverridePolicy,
    (
      conventions::Model,
      events::Model,
      maximum_event_provided_tickets_overrides::Model,
    ),
    maximum_event_provided_tickets_overrides::Model,
  > for MaximumEventProvidedTicketsOverrideGuard
{
  fn new(action: ReadManageAction, model: &maximum_event_provided_tickets_overrides::Model) -> Self
  where
    Self: Sized,
  {
    MaximumEventProvidedTicketsOverrideGuard {
      action,
      model: model.clone(),
    }
  }

  fn get_action(&self) -> &ReadManageAction {
    &self.action
  }

  fn get_model(&self) -> &maximum_event_provided_tickets_overrides::Model {
    &self.model
  }

  async fn get_resource(
    &self,
    model: &maximum_event_provided_tickets_overrides::Model,
    ctx: &Context<'_>,
  ) -> Result<
    (
      conventions::Model,
      events::Model,
      maximum_event_provided_tickets_overrides::Model,
    ),
    Error,
  > {
    let loaders = ctx.data::<Arc<LoaderManager>>()?;
    let event_loader = loaders.maximum_event_provided_tickets_override_event();
    let convention_loader = loaders.event_convention();
    let event_result = event_loader.load_one(model.id).await?;
    let event = event_result.expect_one()?;
    let convention_result = convention_loader.load_one(event.id).await?;
    let convention = convention_result.expect_one()?;

    Ok((convention.clone(), event.clone(), model.clone()))
  }
}

impl
  GuardablePolicy<
    '_,
    (
      conventions::Model,
      events::Model,
      maximum_event_provided_tickets_overrides::Model,
    ),
    maximum_event_provided_tickets_overrides::Model,
  > for MaximumEventProvidedTicketsOverridePolicy
{
  type Guard = MaximumEventProvidedTicketsOverrideGuard;
}
