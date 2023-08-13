use intercode_entities::ticket_types;
use seawater::liquid_drop_impl;
use seawater::model_backed_drop;

use super::drop_context::DropContext;

model_backed_drop!(TicketTypeDrop, ticket_types::Model, DropContext);

#[liquid_drop_impl(i64, DropContext)]
impl TicketTypeDrop {
  fn id(&self) -> i64 {
    self.model.id
  }

  pub fn allows_event_signups(&self) -> bool {
    self.model.allows_event_signups
  }
}
