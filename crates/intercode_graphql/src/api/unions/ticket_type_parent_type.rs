use async_graphql::Union;

use crate::api::merged_objects::{ConventionType, EventType};

#[derive(Union)]
#[graphql(name = "TicketTypeParent")]
pub enum TicketTypeParentType {
  Convention(ConventionType),
  Event(EventType),
}
