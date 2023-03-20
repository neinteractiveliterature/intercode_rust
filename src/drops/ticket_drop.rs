use intercode_entities::tickets;
use seawater::liquid_drop_impl;
use seawater::{belongs_to_related, model_backed_drop, DropError};

use super::{drop_context::DropContext, TicketTypeDrop};

model_backed_drop!(TicketDrop, tickets::Model, DropContext);

#[belongs_to_related(ticket_type, TicketTypeDrop)]
#[liquid_drop_impl(i64, DropContext)]
impl TicketDrop {
  fn id(&self) -> i64 {
    self.model.id
  }

  pub async fn allows_event_signups(&self) -> Result<bool, DropError> {
    let ticket_type_result = self.ticket_type().await;
    let ticket_type = ticket_type_result.get_inner();
    Ok(**ticket_type.allows_event_signups().await.get_inner())
  }
}
