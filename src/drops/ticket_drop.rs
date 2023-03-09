use intercode_entities::tickets;
use seawater::{belongs_to_related, model_backed_drop, DropError};
use seawater::{liquid_drop_impl, liquid_drop_struct};

use super::{drop_context::DropContext, TicketTypeDrop};

model_backed_drop!(TicketDrop, tickets::Model, DropContext);

#[belongs_to_related(ticket_type, TicketTypeDrop)]
#[liquid_drop_impl(i64)]
impl TicketDrop {
  fn id(&self) -> i64 {
    self.model.id
  }

  pub async fn allows_event_signups(&self) -> Result<bool, DropError> {
    let ticket_type = self.ticket_type().await.expect_inner();
    Ok(*ticket_type.allows_event_signups().await.expect_inner())
  }
}
