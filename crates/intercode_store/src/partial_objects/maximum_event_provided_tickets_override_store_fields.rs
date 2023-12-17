use std::sync::Arc;

use crate::{objects::TicketTypeType, policies::MaximumEventProvidedTicketsOverridePolicy};
use async_graphql::*;
use intercode_entities::maximum_event_provided_tickets_overrides;
use intercode_graphql_core::{
  load_one_by_model_id, loader_result_to_required_single, model_backed_type, ModelBackedType,
};
use intercode_graphql_loaders::LoaderManager;
use intercode_policies::{ModelBackedTypeGuardablePolicy, ReadManageAction};
use seawater::loaders::ExpectModel;

use super::EventStoreFields;

model_backed_type!(
  MaximumEventProvidedTicketsOverrideStoreFields,
  maximum_event_provided_tickets_overrides::Model
);

impl MaximumEventProvidedTicketsOverrideStoreFields {
  pub async fn event(&self, ctx: &Context<'_>) -> Result<EventStoreFields> {
    let loader_result =
      load_one_by_model_id!(maximum_event_provided_tickets_override_event, ctx, self)?;
    Ok(loader_result_to_required_single!(
      loader_result,
      EventStoreFields
    ))
  }
}

#[Object(
  guard = "MaximumEventProvidedTicketsOverridePolicy::model_guard(ReadManageAction::Read, self)"
)]
impl MaximumEventProvidedTicketsOverrideStoreFields {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "override_value")]
  async fn override_value(&self) -> i32 {
    self.model.override_value
  }

  #[graphql(name = "ticket_type")]
  async fn ticket_type(&self, ctx: &Context<'_>) -> Result<TicketTypeType> {
    let ticket_type_result = ctx
      .data::<Arc<LoaderManager>>()?
      .maximum_event_provided_tickets_override_ticket_type()
      .load_one(self.model.id)
      .await?;
    let ticket_type = ticket_type_result.expect_one()?;

    Ok(TicketTypeType::new(ticket_type.clone()))
  }
}
