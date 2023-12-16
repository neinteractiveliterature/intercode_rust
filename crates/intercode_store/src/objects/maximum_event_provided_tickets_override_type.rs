use std::sync::Arc;

use crate::{objects::TicketTypeType, policies::MaximumEventProvidedTicketsOverridePolicy};
use async_graphql::*;
use intercode_entities::maximum_event_provided_tickets_overrides;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_graphql_loaders::LoaderManager;
use intercode_policies::{ModelBackedTypeGuardablePolicy, ReadManageAction};
use seawater::loaders::ExpectModel;

model_backed_type!(
  MaximumEventProvidedTicketsOverrideType,
  maximum_event_provided_tickets_overrides::Model
);

#[Object(
  name = "MaximumEventProvidedTicketsOverride",
  guard = "MaximumEventProvidedTicketsOverridePolicy::model_guard(ReadManageAction::Read, self)"
)]
impl MaximumEventProvidedTicketsOverrideType {
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
