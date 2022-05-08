use super::objects::EventType;
use crate::api::objects::ModelBackedType;
use crate::entities::events;
use crate::entity_relay_connection::RelayConnectable;
use crate::SchemaData;
use async_graphql::connection::{query, Connection};
use async_graphql::*;
use sea_orm::EntityTrait;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
  /// Returns the sum of a and b
  async fn add(&self, a: i32, b: i32) -> i32 {
    a + b
  }

  async fn events(
    &self,
    ctx: &Context<'_>,
    after: Option<String>,
    before: Option<String>,
    first: Option<i32>,
    last: Option<i32>,
  ) -> Result<Connection<usize, EventType>> {
    query(
      after,
      before,
      first,
      last,
      |after, before, first, last| async move {
        let db = ctx.data::<SchemaData>()?.db.as_ref();

        let connection = events::Entity::find()
          .relay_connection(
            db,
            |model| EventType::new(model),
            after,
            before,
            first,
            last,
          )
          .to_connection()
          .await?;

        Ok::<_, Error>(connection)
      },
    )
    .await
  }
}
