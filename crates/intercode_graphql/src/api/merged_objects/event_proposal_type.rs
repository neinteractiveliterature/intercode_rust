use async_graphql::*;
use intercode_entities::event_proposals;
use intercode_events::partial_objects::EventProposalEventsFields;
use intercode_forms::partial_objects::EventProposalFormsFields;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_policies::{
  policies::{EventProposalAction, EventProposalPolicy},
  ModelBackedTypeGuardablePolicy,
};

use crate::{api::objects::UserConProfileType, merged_model_backed_type};

use super::{EventCategoryType, EventType};
model_backed_type!(EventProposalGlueFields, event_proposals::Model);

#[Object(guard = "EventProposalPolicy::model_guard(EventProposalAction::Read, self)")]
impl EventProposalGlueFields {
  #[graphql(name = "event")]
  async fn event(&self, ctx: &Context<'_>) -> Result<Option<EventType>> {
    EventProposalEventsFields::from_type(self.clone())
      .event(ctx)
      .await
      .map(|opt| opt.map(EventType::from_type))
  }

  #[graphql(name = "event_category")]
  async fn event_category(&self, ctx: &Context<'_>) -> Result<EventCategoryType> {
    EventProposalEventsFields::from_type(self.clone())
      .event_category(ctx)
      .await
      .map(EventCategoryType::from_type)
  }
  async fn owner(&self, ctx: &Context<'_>) -> Result<UserConProfileType> {
    EventProposalEventsFields::from_type(self.clone())
      .owner(ctx)
      .await
      .map(UserConProfileType::new)
  }
}

merged_model_backed_type!(
  EventProposalType,
  event_proposals::Model,
  "EventProposal",
  EventProposalGlueFields,
  EventProposalFormsFields
);
