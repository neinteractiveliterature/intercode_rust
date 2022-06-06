use super::objects::EventType;
use crate::api::objects::ModelBackedType;
use crate::entities::events;
use crate::entity_relay_connection::RelayConnectable;
use crate::liquid_extensions::build_liquid_parser;
use crate::{QueryData, SchemaData};
use async_graphql::connection::{query, Connection};
use async_graphql::*;
use sea_orm::EntityTrait;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
  async fn preview_liquid(&self, ctx: &Context<'_>, content: String) -> Result<String, Error> {
    let schema_data = ctx.data::<SchemaData>()?;
    let query_data = ctx.data::<QueryData>()?;

    let parser = build_liquid_parser(schema_data, query_data)?;
    let template = parser.parse(content.as_str())?;

    let globals = liquid::object!({
      "num": 4f64,
      "timespan": liquid::object!({}),
      "convention": query_data.convention
    });

    let result = template.render(&globals);

    match result {
      Ok(content) => Ok(content),
      Err(error) => Err(async_graphql::Error::new(error.to_string())),
    }
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
