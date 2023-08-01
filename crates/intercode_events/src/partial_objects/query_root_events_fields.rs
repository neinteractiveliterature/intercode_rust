use async_graphql::{Context, Result};
use intercode_entities::events;
use intercode_graphql_core::{
  entity_relay_connection::{RelayConnectable, RelayConnectionWrapper},
  query_data::QueryData,
  ModelBackedType,
};
use sea_orm::EntityTrait;
use seawater::ConnectionWrapper;

use super::EventEventsFields;

pub struct QueryRootEventsFields;

impl QueryRootEventsFields {
  pub async fn events<'a>(
    ctx: &'a Context<'_>,
    after: Option<u64>,
    before: Option<u64>,
    first: Option<usize>,
    last: Option<usize>,
  ) -> Result<RelayConnectionWrapper<'a, ConnectionWrapper, events::Model, EventEventsFields>> {
    let db = ctx.data::<QueryData>()?.db();

    Ok(events::Entity::find().relay_connection(
      db,
      Box::new(|m: events::Model| EventEventsFields::new(m)),
      after,
      before,
      first,
      last,
    ))
  }
}
