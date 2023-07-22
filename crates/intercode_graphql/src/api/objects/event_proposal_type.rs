use std::sync::Arc;

use async_graphql::*;
use intercode_entities::event_proposals;
use intercode_forms::partial_objects::EventProposalFormsFields;
use intercode_graphql_core::{
  load_one_by_model_id, loader_result_to_optional_single, loader_result_to_required_single,
  model_backed_type, objects::ActiveStorageAttachmentType, scalars::DateScalar, ModelBackedType,
};
use intercode_graphql_loaders::LoaderManager;
use intercode_policies::{
  policies::{EventProposalAction, EventProposalPolicy},
  ModelBackedTypeGuardablePolicy,
};

use super::{EventCategoryType, EventType, RegistrationPolicyType, UserConProfileType};
model_backed_type!(EventProposalApiFields, event_proposals::Model);

#[Object(guard = "EventProposalPolicy::model_guard(EventProposalAction::Read, self)")]
impl EventProposalApiFields {
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

  #[graphql(name = "event")]
  async fn event(&self, ctx: &Context<'_>) -> Result<Option<EventType>> {
    let loader_result = load_one_by_model_id!(event_proposal_event, ctx, self)?;
    Ok(loader_result_to_optional_single!(loader_result, EventType))
  }

  #[graphql(name = "event_category")]
  async fn event_category(&self, ctx: &Context<'_>) -> Result<EventCategoryType> {
    let loader_result = load_one_by_model_id!(event_proposal_event_category, ctx, self)?;
    Ok(loader_result_to_required_single!(
      loader_result,
      EventCategoryType
    ))
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

  async fn owner(&self, ctx: &Context<'_>) -> Result<UserConProfileType> {
    let loader_result = load_one_by_model_id!(event_proposal_owner, ctx, self)?;
    Ok(loader_result_to_required_single!(
      loader_result,
      UserConProfileType
    ))
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

#[derive(MergedObject)]
#[graphql(name = "EventProposal")]
pub struct EventProposalType(EventProposalApiFields, EventProposalFormsFields);

impl ModelBackedType for EventProposalType {
  type Model = event_proposals::Model;

  fn new(model: Self::Model) -> Self {
    Self(
      EventProposalApiFields::new(model.clone()),
      EventProposalFormsFields::new(model),
    )
  }

  fn get_model(&self) -> &Self::Model {
    self.0.get_model()
  }

  fn into_model(self) -> Self::Model {
    self.0.into_model()
  }
}
