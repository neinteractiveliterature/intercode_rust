use async_trait::async_trait;
use intercode_entities::{
  conventions, event_categories, events, model_ext::form_item_permissions::FormItemRole,
};
use sea_orm::{
  sea_query::{Cond, Expr},
  ColumnTrait, DbErr, EntityTrait, QueryFilter, QuerySelect,
};

use crate::{AuthorizationInfo, EntityPolicy, FormResponsePolicy, Policy, ReadManageAction};

use super::has_schedule_release_permissions;

#[derive(PartialEq, Eq)]
pub enum EventAction {
  Read,
  ReadAdminNotes,
  ReadSignups,
  ReadSignupDetails,
  UpdateAdminNotes,
  Drop,
  Create,
  Restore,
  Update,
  ProvideTickets,
  OverrideMaximumEventProvidedTickets,
}

impl From<ReadManageAction> for EventAction {
  fn from(action: ReadManageAction) -> Self {
    match action {
      ReadManageAction::Read => Self::Read,
      ReadManageAction::Manage => Self::Update,
    }
  }
}

async fn has_applicable_permission(
  principal: &AuthorizationInfo,
  permission: &str,
  event: &events::Model,
) -> Result<bool, DbErr> {
  Ok(
    principal
      .has_convention_permission(permission, event.convention_id)
      .await?
      || principal
        .has_event_category_permission(permission, event.convention_id, event.event_category_id)
        .await?,
  )
}

pub struct EventPolicy;

#[async_trait]
impl Policy<AuthorizationInfo, (conventions::Model, events::Model)> for EventPolicy {
  type Action = EventAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &Self::Action,
    (convention, event): &(conventions::Model, events::Model),
  ) -> Result<bool, Self::Error> {
    if !principal.can_act_in_convention(convention.id) {
      return Ok(false);
    }

    match action {
      EventAction::Read => Ok(
        (principal.has_scope("read_events")
          && (convention.site_mode == "single_event"
            || principal
              .team_member_event_ids_in_convention(event.convention_id)
              .await?
              .contains(&event.id)
            || (event.status == "active"
              && has_schedule_release_permissions(
                principal,
                convention,
                &convention.show_event_list,
              )
              .await)
            || has_applicable_permission(principal, "read_inactive_events", event).await?
            || has_applicable_permission(principal, "update_events", event).await?))
          || principal.site_admin_read(),
      ),
      EventAction::ReadSignups => {
        Self::action_permitted(
          principal,
          &EventAction::ReadSignupDetails,
          &(convention.clone(), event.clone()),
        )
        .await
      }
      EventAction::ReadSignupDetails => Ok(
        principal
          .has_scope_and_convention_permission(
            "read_conventions",
            "read_signup_details",
            event.convention_id,
          )
          .await?
          || (principal.has_scope("read_events")
            && principal
              .team_member_event_ids_in_convention(event.convention_id)
              .await?
              .contains(&event.convention_id))
          || principal.site_admin_read(),
      ),
      EventAction::ReadAdminNotes => Ok(
        (principal.has_scope("read_events")
          && has_applicable_permission(principal, "access_admin_notes", event).await?)
          || principal.site_admin_read(),
      ),
      EventAction::UpdateAdminNotes => Ok(
        (principal.has_scope("manage_events")
          && has_applicable_permission(principal, "access_admin_notes", event).await?)
          || principal.site_admin_manage(),
      ),
      EventAction::Drop | EventAction::Create | EventAction::Restore => Ok(
        (principal.has_scope("manage_events")
          && has_applicable_permission(principal, "update_events", event).await?)
          || principal.site_admin_manage(),
      ),
      EventAction::Update => Ok(
        principal.has_scope("manage_events")
          && (principal
            .team_member_event_ids_in_convention(event.convention_id)
            .await?
            .contains(&event.id)
            || principal
              .has_event_category_permission(
                "update_events",
                event.convention_id,
                event.event_category_id,
              )
              .await?
            || principal
              .has_convention_permission("update_events", event.convention_id)
              .await?
            || principal.site_admin_manage()),
      ),
      EventAction::ProvideTickets => todo!(),
      EventAction::OverrideMaximumEventProvidedTickets => Ok(
        principal.has_scope("manage_events")
          && (principal
            .has_convention_permission("override_event_tickets", convention.id)
            .await?
            || principal
              .has_event_category_permission(
                "override_event_tickets",
                convention.id,
                event.event_category_id,
              )
              .await?)
          || principal.site_admin_manage(),
      ),
    }
  }
}

#[async_trait]
impl FormResponsePolicy<AuthorizationInfo, (conventions::Model, events::Model)> for EventPolicy {
  async fn form_item_viewer_role(
    principal: &AuthorizationInfo,
    (_convention, form_response): &(conventions::Model, events::Model),
  ) -> FormItemRole {
    if principal
      .has_convention_permission("update_events", form_response.convention_id)
      .await
      .unwrap_or(false)
      || principal.site_admin_manage()
    {
      return FormItemRole::Admin;
    }

    if principal
      .team_member_event_ids_in_convention(form_response.convention_id)
      .await
      .unwrap_or_default()
      .contains(&form_response.id)
    {
      return FormItemRole::TeamMember;
    }

    if principal
      .active_signups_in_convention_by_event_id(form_response.convention_id)
      .await
      .unwrap_or_default()
      .get(&form_response.id)
      .map(|signups| signups.iter().any(|signup| signup.state == "confirmed"))
      .unwrap_or(false)
    {
      return FormItemRole::ConfirmedAttendee;
    }

    FormItemRole::Normal
  }

  async fn form_item_writer_role(
    principal: &AuthorizationInfo,
    resource: &(conventions::Model, events::Model),
  ) -> FormItemRole {
    Self::form_item_viewer_role(principal, resource).await
  }
}

impl EntityPolicy<AuthorizationInfo, events::Model> for EventPolicy {
  type Action = EventAction;

  fn accessible_to(
    principal: &AuthorizationInfo,
    action: &Self::Action,
  ) -> sea_orm::Select<events::Entity> {
    let scope = events::Entity::find();

    // TODO consider implementing other actions
    if *action != EventAction::Read {
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
          .filter(events::Column::ConventionId.eq(profile.convention_id))
      })
      .unwrap_or(scope);

    if principal.site_admin_read() {
      return scope;
    }

    let scope = scope.left_join(conventions::Entity).filter(
      Cond::any()
        .add(
          events::Column::Id.in_subquery(
            QuerySelect::query(
              &mut principal
                .events_where_team_member()
                .select_only()
                .column(events::Column::Id),
            )
            .take(),
          ),
        )
        .add(conventions::Column::SiteMode.eq("single_event"))
        .add(
          events::Column::ConventionId.in_subquery(
            QuerySelect::query(
              &mut principal
                .conventions_with_permissions(&["read_inactive_events", "update_events"])
                .select_only()
                .column(conventions::Column::Id),
            )
            .take(),
          ),
        )
        // event updaters can see dropped events in their categories
        .add(
          events::Column::EventCategoryId.in_subquery(
            QuerySelect::query(
              &mut principal
                .event_categories_with_permission("update_events")
                .select_only()
                .column(event_categories::Column::Id),
            )
            .take(),
          ),
        ),
    );

    scope
  }

  fn id_column() -> events::Column {
    events::Column::Id
  }
}
