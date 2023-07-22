use std::sync::Arc;

use async_graphql::{Context, Error};
use axum::async_trait;
use intercode_entities::{conventions, events, team_members, user_con_profiles};
use intercode_graphql_loaders::LoaderManager;
use sea_orm::{
  sea_query::{Cond, Expr},
  ColumnTrait, DbErr, EntityTrait, QueryFilter, QuerySelect,
};
use seawater::loaders::ExpectModel;

use crate::{
  authorization_info::AuthorizationInfo,
  policy::{Policy, ReadManageAction},
  EntityPolicy, GuardablePolicy, PolicyGuard,
};

use super::{EventAction, EventPolicy};

pub struct TeamMemberPolicy;

#[async_trait]
impl Policy<AuthorizationInfo, (conventions::Model, events::Model, team_members::Model)>
  for TeamMemberPolicy
{
  type Action = ReadManageAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &ReadManageAction,
    (convention, event, _team_member): &(conventions::Model, events::Model, team_members::Model),
  ) -> Result<bool, Self::Error> {
    if !principal.can_act_in_convention(event.convention_id) {
      return Ok(false);
    }

    match action {
      ReadManageAction::Read => Ok(
        (principal.has_scope("read_events")
          && EventPolicy::action_permitted(
            principal,
            &EventAction::Read,
            &(convention.clone(), event.clone()),
          )
          .await?)
          || (principal.has_scope("read_conventions")
            && (principal
              .has_convention_permission("update_event_team_members", event.convention_id)
              .await?
              || EventPolicy::action_permitted(
                principal,
                &EventAction::Read,
                &(convention.clone(), event.clone()),
              )
              .await?))
          || principal.site_admin_read(),
      ),
      ReadManageAction::Manage => Ok(
        (principal.has_scope("manage_events")
          && EventPolicy::action_permitted(
            principal,
            &EventAction::Read,
            &(convention.clone(), event.clone()),
          )
          .await?)
          || (principal.has_scope("manage_conventions")
            && (principal
              .has_convention_permission("update_event_team_members", event.convention_id)
              .await?))
          || principal.site_admin_manage(),
      ),
    }
  }
}

impl EntityPolicy<AuthorizationInfo, team_members::Model> for TeamMemberPolicy {
  type Action = ReadManageAction;

  fn accessible_to(
    principal: &AuthorizationInfo,
    action: &Self::Action,
  ) -> sea_orm::Select<team_members::Entity> {
    let scope = team_members::Entity::find();

    // TODO consider implementing other actions
    if *action != ReadManageAction::Read {
      return scope.filter(Expr::cust("1 = 0"));
    }

    if !principal.has_scope("read_events") {
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

    scope.left_join(events::Entity).filter(
      Cond::any()
        .add(
          team_members::Column::EventId.in_subquery(
            QuerySelect::query(
              &mut principal
                .events_where_team_member()
                .select_only()
                .column(events::Column::Id),
            )
            .take(),
          ),
        )
        .add(
          team_members::Column::EventId.in_subquery(
            QuerySelect::query(
              &mut EventPolicy::accessible_to(principal, &EventAction::Read)
                .select_only()
                .column(events::Column::Id),
            )
            .take(),
          ),
        )
        .add(
          events::Column::ConventionId.in_subquery(
            QuerySelect::query(
              &mut principal
                .conventions_with_permission("update_event_team_members")
                .select_only()
                .column(conventions::Column::Id),
            )
            .take(),
          ),
        ),
    )
  }

  fn id_column() -> team_members::Column {
    team_members::Column::Id
  }
}

pub struct TeamMemberGuard {
  action: ReadManageAction,
  model: team_members::Model,
}

#[async_trait]
impl
  PolicyGuard<
    '_,
    TeamMemberPolicy,
    (conventions::Model, events::Model, team_members::Model),
    team_members::Model,
  > for TeamMemberGuard
{
  fn new(action: ReadManageAction, model: &team_members::Model) -> Self
  where
    Self: Sized,
  {
    TeamMemberGuard {
      action,
      model: model.clone(),
    }
  }

  fn get_action(&self) -> &ReadManageAction {
    &self.action
  }

  fn get_model(&self) -> &team_members::Model {
    &self.model
  }

  async fn get_resource(
    &self,
    model: &team_members::Model,
    ctx: &Context<'_>,
  ) -> Result<(conventions::Model, events::Model, team_members::Model), Error> {
    let loaders = ctx.data::<Arc<LoaderManager>>()?;
    let event_loader = loaders.team_member_event();
    let convention_loader = loaders.event_convention();
    let event_result = event_loader.load_one(model.id).await?;
    let event = event_result.expect_one()?;
    let convention_result = convention_loader.load_one(event.id).await?;
    let convention = convention_result.expect_one()?;

    Ok((convention.clone(), event.clone(), model.clone()))
  }
}

impl
  GuardablePolicy<'_, (conventions::Model, events::Model, team_members::Model), team_members::Model>
  for TeamMemberPolicy
{
  type Guard = TeamMemberGuard;
}
