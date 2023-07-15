use async_trait::async_trait;
use intercode_entities::{conventions, events, ticket_types};
use intercode_policies::{AuthorizationInfo, EntityPolicy, Policy, ReadManageAction};
use sea_orm::{sea_query::Expr, DbErr, EntityTrait, QueryFilter};

async fn is_single_event_team_member(
  principal: &AuthorizationInfo,
  single_event: Option<&events::Model>,
) -> Result<bool, DbErr> {
  let Some(single_event) = single_event else {
    return Ok(false);
  };

  Ok(
    principal
      .team_member_event_ids_in_convention(single_event.convention_id)
      .await?
      .contains(&single_event.id),
  )
}

pub struct TicketTypePolicy;

#[async_trait]
impl
  Policy<
    AuthorizationInfo,
    (
      conventions::Model,
      Option<events::Model>,
      ticket_types::Model,
    ),
  > for TicketTypePolicy
{
  type Action = ReadManageAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &ReadManageAction,
    (convention, single_event, _ticket_type): &(
      conventions::Model,
      Option<events::Model>,
      ticket_types::Model,
    ),
  ) -> Result<bool, Self::Error> {
    match action {
      ReadManageAction::Read => Ok(true),
      ReadManageAction::Manage => Ok(
        principal
          .has_scope_and_convention_permission(
            "manage_conventions",
            "update_ticket_types",
            convention.id,
          )
          .await?
          || is_single_event_team_member(principal, single_event.as_ref()).await?
          || principal.site_admin_manage(),
      ),
    }
  }
}

impl EntityPolicy<AuthorizationInfo, ticket_types::Model> for TicketTypePolicy {
  type Action = ReadManageAction;

  fn id_column() -> ticket_types::Column {
    ticket_types::Column::Id
  }

  fn accessible_to(
    _principal: &AuthorizationInfo,
    action: &Self::Action,
  ) -> sea_orm::Select<ticket_types::Entity> {
    match action {
      ReadManageAction::Read => ticket_types::Entity::find(),
      ReadManageAction::Manage => ticket_types::Entity::find().filter(Expr::cust("0 = 1")),
    }
  }
}
