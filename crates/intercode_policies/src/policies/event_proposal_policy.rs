use std::collections::HashSet;

use async_trait::async_trait;
use cached::once_cell::sync::Lazy;
use intercode_entities::{
  conventions, event_proposals,
  model_ext::{event_proposals::EventProposalStatus, form_item_permissions::FormItemRole},
};
use sea_orm::{
  sea_query::{Cond, Expr},
  ColumnTrait, DbErr, EntityTrait, Iterable, QueryFilter, Select,
};

use crate::{
  AuthorizationInfo, CRUDAction, EntityPolicy, FormResponsePolicy, Policy, ReadManageAction,
};

pub enum EventProposalAction {
  Read,
  Create,
  Update,
  Delete,
  ReadAdminNotes,
  UpdateAdminNotes,
  Submit,
}

impl From<ReadManageAction> for EventProposalAction {
  fn from(value: ReadManageAction) -> Self {
    match value {
      ReadManageAction::Read => Self::Read,
      ReadManageAction::Manage => Self::Update,
    }
  }
}

impl From<CRUDAction> for EventProposalAction {
  fn from(value: CRUDAction) -> Self {
    match value {
      CRUDAction::Create => Self::Create,
      CRUDAction::Read => Self::Read,
      CRUDAction::Update => Self::Update,
      CRUDAction::Delete => Self::Delete,
    }
  }
}

static NON_DRAFT_STATUSES: Lazy<HashSet<EventProposalStatus>> = Lazy::new(|| {
  EventProposalStatus::iter()
    .filter(|status| *status != EventProposalStatus::Draft)
    .collect()
});

static NON_PENDING_STATUSES: Lazy<HashSet<EventProposalStatus>> = Lazy::new(|| {
  NON_DRAFT_STATUSES
    .iter()
    .cloned()
    .filter(|status| *status != EventProposalStatus::Proposed)
    .collect()
});

static NON_FINAL_STATUSES: Lazy<HashSet<EventProposalStatus>> = Lazy::new(|| {
  EventProposalStatus::iter()
    .filter(|status| {
      *status != EventProposalStatus::Accepted
        && *status != EventProposalStatus::Rejected
        && *status != EventProposalStatus::Withdrawn
    })
    .collect()
});

fn is_non_draft_event_proposal(event_proposal: &event_proposals::Model) -> bool {
  event_proposal
    .status
    .as_ref()
    .map(|status| NON_DRAFT_STATUSES.contains(status))
    .unwrap_or(false)
}

fn is_non_pending_event_proposal(event_proposal: &event_proposals::Model) -> bool {
  event_proposal
    .status
    .as_ref()
    .map(|status| NON_PENDING_STATUSES.contains(status))
    .unwrap_or(false)
}

fn is_non_final_event_proposal(event_proposal: &event_proposals::Model) -> bool {
  event_proposal
    .status
    .as_ref()
    .map(|status| NON_FINAL_STATUSES.contains(status))
    .unwrap_or(false)
}

async fn is_owner(
  principal: &AuthorizationInfo,
  event_proposal: &event_proposals::Model,
) -> Result<bool, DbErr> {
  let Some(owner_id) = event_proposal.owner_id else {
    return Ok(false);
  };

  Ok(principal.user_con_profile_ids().await?.contains(&owner_id))
}

async fn team_member_for_accepted_proposal(
  principal: &AuthorizationInfo,
  event_proposal: &event_proposals::Model,
) -> Result<bool, DbErr> {
  let Some(convention_id) = event_proposal.convention_id else {
    return Ok(false);
  };

  let Some(event_id) = event_proposal.event_id else {
    return Ok(false);
  };

  Ok(
    principal
      .team_member_event_ids_in_convention(convention_id)
      .await?
      .contains(&event_id),
  )
}

async fn has_applicable_permission(
  permission: &str,
  principal: &AuthorizationInfo,
  convention: &conventions::Model,
  event_proposal: &event_proposals::Model,
) -> Result<bool, DbErr> {
  Ok(
    principal
      .has_convention_permission(permission, convention.id)
      .await?
      || principal
        .has_event_category_permission(permission, convention.id, event_proposal.event_category_id)
        .await?,
  )
}

pub struct EventProposalPolicy;

#[async_trait]
impl Policy<AuthorizationInfo, (conventions::Model, event_proposals::Model)>
  for EventProposalPolicy
{
  type Action = EventProposalAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &Self::Action,
    (convention, event_proposal): &(conventions::Model, event_proposals::Model),
  ) -> Result<bool, Self::Error> {
    if !principal.can_act_in_convention(convention.id) {
      return Ok(false);
    }

    match action {
      EventProposalAction::Read => Ok(
        principal.has_scope("read_events")
          && ((principal.user.is_some() && is_owner(principal, event_proposal).await?)
            || (is_non_pending_event_proposal(event_proposal)
              && has_applicable_permission(
                "read_event_proposals",
                principal,
                convention,
                event_proposal,
              )
              .await?)
            || (is_non_draft_event_proposal(event_proposal)
              && (has_applicable_permission(
                "read_pending_event_proposals",
                principal,
                convention,
                event_proposal,
              )
              .await?
                || principal.site_admin_read()))),
      ),
      EventProposalAction::ReadAdminNotes => Ok(
        (principal.has_scope("read_events")
          && has_applicable_permission("read_admin_notes", principal, convention, event_proposal)
            .await?)
          || principal.site_admin_read(),
      ),
      EventProposalAction::Update => Ok(
        (principal.has_scope("manage_events")
          && ((is_non_final_event_proposal(event_proposal)
            && is_owner(principal, event_proposal).await?)
            || (is_non_draft_event_proposal(event_proposal)
              && has_applicable_permission(
                "update_event_proposals",
                principal,
                convention,
                event_proposal,
              )
              .await?)
            || team_member_for_accepted_proposal(principal, event_proposal).await?))
          || principal.site_admin_manage(),
      ),
      EventProposalAction::Delete => Ok(
        event_proposal.status == Some(EventProposalStatus::Draft)
          && is_owner(principal, event_proposal).await?,
      ),
      _ => todo!(),
    }
  }
}

#[async_trait]
impl FormResponsePolicy<AuthorizationInfo, (conventions::Model, event_proposals::Model)>
  for EventProposalPolicy
{
  async fn form_item_viewer_role(
    principal: &AuthorizationInfo,
    (convention, form_response): &(conventions::Model, event_proposals::Model),
  ) -> FormItemRole {
    if is_non_draft_event_proposal(form_response)
      && principal
        .has_convention_permission("update_event_proposals", convention.id)
        .await
        .unwrap_or(false)
      || principal.site_admin_manage()
    {
      return FormItemRole::Admin;
    }

    if let Some(owner_id) = form_response.owner_id {
      if principal
        .user_con_profile_ids()
        .await
        .cloned()
        .unwrap_or_default()
        .contains(&owner_id)
      {
        return FormItemRole::TeamMember;
      }
    }

    FormItemRole::Normal
  }

  async fn form_item_writer_role(
    principal: &AuthorizationInfo,
    resource: &(conventions::Model, event_proposals::Model),
  ) -> FormItemRole {
    EventProposalPolicy::form_item_viewer_role(principal, resource).await
  }
}

impl EntityPolicy<AuthorizationInfo, event_proposals::Model> for EventProposalPolicy {
  type Action = EventProposalAction;

  fn accessible_to(
    principal: &AuthorizationInfo,
    action: &Self::Action,
  ) -> Select<event_proposals::Entity> {
    match action {
      EventProposalAction::Read => {
        let scope = event_proposals::Entity::find();
        if principal.has_scope("read_events")
          && principal.site_admin()
          && principal.assumed_identity_from_profile.is_none()
        {
          return scope
            .filter(event_proposals::Column::Status.is_in(NON_DRAFT_STATUSES.iter().cloned()));
        }

        scope.filter(
          Cond::any()
            .add_option(
              principal
                .user
                .as_ref()
                .map(|user| event_proposals::Column::OwnerId.eq(user.id)),
            )
            .add(
              event_proposals::Column::EventCategoryId
                .in_subquery(
                  sea_orm::QuerySelect::query(
                    &mut principal.event_categories_with_permission("read_pending_event_proposals"),
                  )
                  .take(),
                )
                .and(event_proposals::Column::Status.is_in(NON_DRAFT_STATUSES.iter().cloned())),
            )
            .add(
              event_proposals::Column::EventCategoryId
                .in_subquery(
                  sea_orm::QuerySelect::query(
                    &mut principal.event_categories_with_permission("read_event_proposals"),
                  )
                  .take(),
                )
                .and(event_proposals::Column::Status.is_in(NON_PENDING_STATUSES.iter().cloned())),
            )
            .add(
              event_proposals::Column::ConventionId
                .in_subquery(
                  sea_orm::QuerySelect::query(
                    &mut principal.conventions_with_permission("read_pending_event_proposals"),
                  )
                  .take(),
                )
                .and(event_proposals::Column::Status.is_in(NON_DRAFT_STATUSES.iter().cloned())),
            )
            .add(
              event_proposals::Column::ConventionId
                .in_subquery(
                  sea_orm::QuerySelect::query(
                    &mut principal.conventions_with_permission("read_event_proposals"),
                  )
                  .take(),
                )
                .and(event_proposals::Column::Status.is_in(NON_PENDING_STATUSES.iter().cloned())),
            ),
        )
      }
      _ => event_proposals::Entity::find().filter(Expr::cust("1 = 0")),
    }
  }
}
