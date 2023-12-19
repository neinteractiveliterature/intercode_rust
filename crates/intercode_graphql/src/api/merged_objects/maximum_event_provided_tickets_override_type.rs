use async_graphql::{Context, Error, Object};
use intercode_entities::maximum_event_provided_tickets_overrides;
use intercode_graphql_core::model_backed_type;
use intercode_policies::{ModelBackedTypeGuardablePolicy, ReadManageAction};
use intercode_store::{
  partial_objects::{
    MaximumEventProvidedTicketsOverrideStoreExtensions,
    MaximumEventProvidedTicketsOverrideStoreFields,
  },
  policies::MaximumEventProvidedTicketsOverridePolicy,
};

use crate::merged_model_backed_type;

use super::{ticket_type_type::TicketTypeType, EventType};

model_backed_type!(
  MaximumEventProvidedTicketsOverrideGlueFields,
  maximum_event_provided_tickets_overrides::Model
);

impl MaximumEventProvidedTicketsOverrideStoreExtensions
  for MaximumEventProvidedTicketsOverrideGlueFields
{
}

#[Object(
  guard = "MaximumEventProvidedTicketsOverridePolicy::model_guard(ReadManageAction::Read, self)"
)]
impl MaximumEventProvidedTicketsOverrideGlueFields {
  async fn event(&self, ctx: &Context<'_>) -> Result<EventType, Error> {
    MaximumEventProvidedTicketsOverrideStoreExtensions::event(self, ctx).await
  }

  #[graphql(name = "ticket_type")]
  async fn ticket_type(&self, ctx: &Context<'_>) -> Result<TicketTypeType, Error> {
    MaximumEventProvidedTicketsOverrideStoreExtensions::ticket_type(self, ctx).await
  }
}

merged_model_backed_type!(
  MaximumEventProvidedTicketsOverrideType,
  maximum_event_provided_tickets_overrides::Model,
  "MaximumEventProvidedTicketsOverride",
  MaximumEventProvidedTicketsOverrideGlueFields,
  MaximumEventProvidedTicketsOverrideStoreFields
);
