use intercode_entities::ticket_types;
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
use seawater::model_backed_drop;

model_backed_drop!(TicketTypeDrop, ticket_types::Model);

#[liquid_drop_impl]
impl TicketTypeDrop {
  pub fn id(&self) -> i64 {
    self.model.id
  }

  pub fn allows_event_signups(&self) -> bool {
    self.model.allows_event_signups
  }
}
