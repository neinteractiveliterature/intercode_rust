use std::sync::Arc;

use async_graphql::*;
use intercode_entities::{event_proposals, user_con_profiles};
use intercode_graphql_core::{
  load_one_by_model_id, loader_result_to_optional_single, loader_result_to_required_single,
  model_backed_type, objects::ActiveStorageAttachmentType, scalars::DateScalar, ModelBackedType,
};
use intercode_graphql_loaders::LoaderManager;
use intercode_policies::{
  policies::{EventProposalAction, EventProposalPolicy},
  ModelBackedTypeGuardablePolicy,
};
use seawater::loaders::ExpectModel;

use crate::objects::RegistrationPolicyType;

use super::{EventCategoryEventsFields, EventEventsFields};

model_backed_type!(EventProposalEventsFields, event_proposals::Model);

impl EventProposalEventsFields {
  pub async fn event(&self, ctx: &Context<'_>) -> Result<Option<EventEventsFields>> {
    let loader_result = load_one_by_model_id!(event_proposal_event, ctx, self)?;
    Ok(loader_result_to_optional_single!(
      loader_result,
      EventEventsFields
    ))
  }

  pub async fn event_category(&self, ctx: &Context<'_>) -> Result<EventCategoryEventsFields> {
    let loader_result = load_one_by_model_id!(event_proposal_event_category, ctx, self)?;
    Ok(loader_result_to_required_single!(
      loader_result,
      EventCategoryEventsFields
    ))
  }

  pub async fn owner(&self, ctx: &Context<'_>) -> Result<user_con_profiles::Model> {
    let loader_result = load_one_by_model_id!(event_proposal_owner, ctx, self)?;
    Ok(loader_result.expect_one()?.clone())
  }
}

#[Object(guard = "EventProposalPolicy::model_guard(EventProposalAction::Read, self)")]
impl EventProposalEventsFields {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(
    name = "admin_notes",
    guard = "EventProposalPolicy::model_guard(EventProposalAction::ReadAdminNotes, self)"
  )]
  async fn admin_notes(&self) -> Option<&str> {
    self.model.admin_notes.as_deref()
  }

  async fn images(&self, ctx: &Context<'_>) -> Result<Vec<ActiveStorageAttachmentType>> {
    let blobs = ctx
      .data::<Arc<LoaderManager>>()?
      .event_proposal_attached_images
      .load_one(self.model.id)
      .await?
      .unwrap_or_default();

    Ok(
      blobs
        .into_iter()
        .map(ActiveStorageAttachmentType::new)
        .collect(),
    )
  }

  #[graphql(name = "length_seconds")]
  async fn length_seconds(&self) -> Option<i32> {
    self.model.length_seconds
  }

  #[graphql(name = "registration_policy")]
  async fn registration_policy(&self) -> RegistrationPolicyType {
    RegistrationPolicyType(
      self
        .model
        .registration_policy
        .as_ref()
        .map(|json| serde_json::from_value(json.clone()).unwrap_or_default())
        .unwrap_or_default(),
    )
  }

  async fn status(&self) -> Option<String> {
    self.model.status.as_ref().map(|status| {
      serde_json::to_value(status)
        .unwrap()
        .as_str()
        .unwrap()
        .to_string()
    })
  }

  #[graphql(name = "submitted_at")]
  async fn submitted_at(&self) -> Result<Option<DateScalar>> {
    self
      .model
      .submitted_at
      .map(DateScalar::try_from)
      .transpose()
  }

  async fn title(&self) -> Option<&str> {
    self.model.title.as_deref()
  }

  #[graphql(name = "updated_at")]
  async fn updated_at(&self) -> Result<DateScalar> {
    self.model.updated_at.try_into()
  }
}
