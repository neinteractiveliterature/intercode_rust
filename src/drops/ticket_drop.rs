use intercode_entities::tickets;
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
use seawater::{belongs_to_related, model_backed_drop, DropError};

use super::TicketTypeDrop;

model_backed_drop!(TicketDrop, tickets::Model);

#[belongs_to_related(ticket_type, TicketTypeDrop)]
#[liquid_drop_impl]
impl TicketDrop {
  pub fn id(&self) -> i64 {
    self.model.id
  }

  pub async fn allows_event_signups(&self) -> Result<bool, DropError> {
    self
      .ticket_type()
      .await
      .map(|ticket_type| ticket_type.allows_event_signups())
  }
}
