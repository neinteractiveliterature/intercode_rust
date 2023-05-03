use async_graphql::*;
use chrono::NaiveDateTime;
use intercode_entities::event_proposals;

use crate::{load_one_by_model_id, loader_result_to_required_single, model_backed_type};

use super::{EventCategoryType, ModelBackedType, RegistrationPolicyType, UserConProfileType};
model_backed_type!(EventProposalType, event_proposals::Model);

#[Object(name = "EventProposal")]
impl EventProposalType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "event_category")]
  async fn event_category(&self, ctx: &Context<'_>) -> Result<EventCategoryType> {
    let loader_result = load_one_by_model_id!(event_proposal_event_category, ctx, self)?;
    Ok(loader_result_to_required_single!(
      loader_result,
      EventCategoryType
    ))
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
    self.model.status.as_ref().map(|status| status.to_string())
  }

  #[graphql(name = "submitted_at")]
  async fn submitted_at(&self) -> Option<&NaiveDateTime> {
    self.model.submitted_at.as_ref()
  }

  async fn title(&self) -> Option<&str> {
    self.model.title.as_deref()
  }

  #[graphql(name = "updated_at")]
  async fn updated_at(&self) -> &NaiveDateTime {
    &self.model.updated_at
  }
}
