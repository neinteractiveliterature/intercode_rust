use async_graphql::{Context, Error, Object};
use intercode_entities::maximum_event_provided_tickets_overrides;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_policies::{ModelBackedTypeGuardablePolicy, ReadManageAction};
use intercode_store::{
  partial_objects::MaximumEventProvidedTicketsOverrideStoreFields,
  policies::MaximumEventProvidedTicketsOverridePolicy,
};

use crate::merged_model_backed_type;

use super::EventType;

model_backed_type!(
  MaximumEventProvidedTicketsOverrideGlueFields,
  maximum_event_provided_tickets_overrides::Model
);

#[Object(
  guard = "MaximumEventProvidedTicketsOverridePolicy::model_guard(ReadManageAction::Read, self)"
)]
impl MaximumEventProvidedTicketsOverrideGlueFields {
  pub async fn event(&self, ctx: &Context<'_>) -> Result<EventType, Error> {
    MaximumEventProvidedTicketsOverrideStoreFields::from_type(self.clone())
      .event(ctx)
      .await
      .map(EventType::from_type)
  }
}

merged_model_backed_type!(
  MaximumEventProvidedTicketsOverrideType,
  maximum_event_provided_tickets_overrides::Model,
  "MaximumEventProvidedTicketsOverride",
  MaximumEventProvidedTicketsOverrideGlueFields,
  MaximumEventProvidedTicketsOverrideStoreFields
);
