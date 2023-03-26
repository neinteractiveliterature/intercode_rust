use async_graphql::*;
use intercode_entities::tickets;
use seawater::loaders::ExpectModels;

use crate::{model_backed_type, QueryData};

use super::{EventType, ModelBackedType, TicketTypeType, UserConProfileType};
model_backed_type!(TicketType, tickets::Model);

#[Object(name = "Ticket")]
impl TicketType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "provided_by_event")]
  async fn provided_by_event(&self, ctx: &Context<'_>) -> Result<Option<EventType>> {
    let loader = ctx
      .data::<QueryData>()?
      .loaders()
      .ticket_provided_by_event();

    Ok(
      loader
        .load_one(self.model.id)
        .await?
        .try_one()
        .map(|event| EventType::new(event.clone())),
    )
  }

  #[graphql(name = "ticket_type")]
  async fn ticket_type(&self, ctx: &Context<'_>) -> Result<TicketTypeType> {
    let loader = ctx.data::<QueryData>()?.loaders().ticket_ticket_type();
    loader
      .load_one(self.model.id)
      .await?
      .expect_one()
      .map(|ticket_type| TicketTypeType::new(ticket_type.clone()))
  }

  #[graphql(name = "user_con_profile")]
  async fn user_con_profile(&self, ctx: &Context<'_>) -> Result<UserConProfileType> {
    let loader = ctx.data::<QueryData>()?.loaders().ticket_user_con_profile();
    loader
      .load_one(self.model.id)
      .await?
      .expect_one()
      .map(|user_con_profile| UserConProfileType::new(user_con_profile.clone()))
  }
}
