use super::{ModelBackedType, TicketTypeType};
use crate::{model_backed_type, policy_guard::PolicyGuard, QueryData};
use async_graphql::*;
use intercode_entities::{conventions, events, maximum_event_provided_tickets_overrides};
use intercode_policies::{policies::MaximumEventProvidedTicketsOverridePolicy, ReadManageAction};
use seawater::loaders::ExpectModel;

model_backed_type!(
  MaximumEventProvidedTicketsOverrideType,
  maximum_event_provided_tickets_overrides::Model
);

impl MaximumEventProvidedTicketsOverrideType {
  fn policy_guard(
    &self,
    action: ReadManageAction,
  ) -> PolicyGuard<
    '_,
    MaximumEventProvidedTicketsOverridePolicy,
    (
      conventions::Model,
      events::Model,
      maximum_event_provided_tickets_overrides::Model,
    ),
    maximum_event_provided_tickets_overrides::Model,
  > {
    PolicyGuard::new(action, &self.model, move |model, ctx| {
      let model = model.clone();
      let ctx = ctx;
      let query_data = ctx.data::<QueryData>();

      Box::pin(async {
        let query_data = query_data?;
        let event_loader = query_data
          .loaders()
          .maximum_event_provided_tickets_override_event();
        let convention_loader = query_data.loaders().event_convention();
        let event_result = event_loader.load_one(model.id).await?;
        let event = event_result.expect_one()?;
        let convention_result = convention_loader.load_one(event.id).await?;
        let convention = convention_result.expect_one()?;

        Ok((convention.clone(), event.clone(), model))
      })
    })
  }
}

#[Object(
  name = "MaximumEventProvidedTicketsOverride",
  guard = "self.policy_guard(ReadManageAction::Read)"
)]
impl MaximumEventProvidedTicketsOverrideType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "override_value")]
  async fn override_value(&self) -> Option<i32> {
    self.model.override_value
  }

  #[graphql(name = "ticket_type")]
  async fn ticket_type(&self, ctx: &Context<'_>) -> Result<TicketTypeType> {
    let ticket_type_result = ctx
      .data::<QueryData>()?
      .loaders()
      .maximum_event_provided_tickets_override_ticket_type()
      .load_one(self.model.id)
      .await?;
    let ticket_type = ticket_type_result.expect_one()?;

    Ok(TicketTypeType::new(ticket_type.clone()))
  }
}
