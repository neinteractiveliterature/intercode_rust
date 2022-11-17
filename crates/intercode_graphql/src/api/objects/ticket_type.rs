use async_graphql::*;
use intercode_entities::tickets;

use crate::model_backed_type;
model_backed_type!(TicketType, tickets::Model);

#[Object(name = "Ticket")]
impl TicketType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }
}
