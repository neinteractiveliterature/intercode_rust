use super::objects::{ConventionType, EventType};
use crate::api::objects::ModelBackedType;
use crate::entity_relay_connection::RelayConnectable;
use crate::{QueryData, SchemaData};
use async_graphql::connection::{query, Connection};
use async_graphql::*;
use intercode_entities::events;
use liquid::object;
use sea_orm::EntityTrait;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
  async fn convention_by_request_host(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<ConventionType>, Error> {
    let query_data = ctx.data::<QueryData>()?;

    match query_data.convention.as_ref() {
      Some(convention) => Ok(Some(ConventionType::new(convention.to_owned()))),
      None => Ok(None),
    }
  }

  async fn preview_liquid(&self, ctx: &Context<'_>, content: String) -> Result<String, Error> {
    let schema_data = ctx.data::<SchemaData>()?;
    let query_data = ctx.data::<QueryData>()?;
    query_data
      .render_liquid(schema_data, content.as_str(), object!({}), None)
      .await
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
          .relay_connection(db, EventType::new, after, before, first, last)
          .to_connection()
          .await?;

        Ok::<_, Error>(connection)
      },
    )
    .await
  }
}