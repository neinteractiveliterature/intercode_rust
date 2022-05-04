use super::entities::conventions;
use super::entities::events;
use crate::SchemaData;

use async_graphql::types::connection::Connection;
use async_graphql::types::connection::*;
use async_graphql::*;
use sea_orm::query::*;
use sea_orm::EntityTrait;
use sea_orm::FromQueryResult;

pub struct ConventionType {
  model: conventions::Model,
}

#[Object]
impl ConventionType {
  async fn id(&self) -> ID {
    ID(self.model.id.to_string())
  }

  async fn name(&self) -> &Option<String> {
    &self.model.name
  }
}

pub struct EventType {
  model: events::Model,
}

#[Object]
impl EventType {
  async fn id(&self) -> ID {
    ID(self.model.id.to_string())
  }

  async fn title(&self) -> &String {
    &self.model.title
  }

  async fn author(&self) -> &Option<String> {
    &self.model.author
  }

  async fn email(&self) -> &Option<String> {
    &self.model.email
  }

  async fn convention(&self, ctx: &Context<'_>) -> Result<Option<ConventionType>, Error> {
    let loader = &ctx.data::<SchemaData>()?.convention_id_loader;

    if let Some(convention_id) = self.model.convention_id {
      let model = loader.load_one(convention_id).await?;
      if let Some(model) = model {
        Ok(Some(ConventionType { model }))
      } else {
        Err(Error::new(format!(
          "Convention {} not found",
          convention_id
        )))
      }
    } else {
      Ok(None)
    }
  }
}

pub trait RelayConnectable<
  'db,
  E: EntityTrait<Model = M>,
  M: FromQueryResult + Sized + Send + Sync + 'db,
  G,
  F: Fn(M) -> G,
>
{
  fn relay_connection<C: ConnectionTrait>(
    self: Self,
    db: &'db C,
    to_graphql_representation: F,
    after: Option<usize>,
    before: Option<usize>,
    first: Option<usize>,
    last: Option<usize>,
  ) -> RelayConnectionWrapper<'db, C, M, E, G, F>;
}

impl<
    'db,
    E: EntityTrait<Model = M>,
    M: FromQueryResult + Sized + Send + Sync + 'db,
    G,
    F: Fn(M) -> G,
  > RelayConnectable<'db, E, M, G, F> for Select<E>
{
  fn relay_connection<C: ConnectionTrait>(
    self: Self,
    db: &'db C,
    to_graphql_representation: F,
    after: Option<usize>,
    before: Option<usize>,
    first: Option<usize>,
    last: Option<usize>,
  ) -> RelayConnectionWrapper<'db, C, M, E, G, F> {
    RelayConnectionWrapper {
      select: self,
      db,
      to_graphql_representation,
      after,
      before,
      first,
      last,
    }
  }
}

pub struct RelayConnectionWrapper<'db, C, M, E, G, F>
where
  C: ConnectionTrait,
  M: FromQueryResult + Sized + Send + Sync + 'db,
  E: EntityTrait<Model = M>,
  F: Fn(M) -> G,
{
  select: Select<E>,
  db: &'db C,
  to_graphql_representation: F,
  after: Option<usize>,
  before: Option<usize>,
  first: Option<usize>,
  last: Option<usize>,
}

impl<'db, C, M, E, G, F> RelayConnectionWrapper<'db, C, M, E, G, F>
where
  C: ConnectionTrait,
  M: FromQueryResult + Sized + Send + Sync + 'db,
  E: EntityTrait<Model = M>,
  F: Fn(M) -> G + Copy,
{
  async fn total_count(self: &Self) -> Result<usize, sea_orm::DbErr> {
    self
      .select
      .clone()
      .into_model::<M>()
      .paginate(self.db, 1)
      .num_items()
      .await
  }

  async fn to_connection(
    self: &Self,
  ) -> Result<async_graphql::connection::Connection<usize, G>, sea_orm::DbErr> {
    let iter = self.to_graphql_representation;
    let db = self.db;

    let total = self.total_count().await?;

    let mut start = self.after.map(|after| after + 1).unwrap_or(0);
    let end = std::cmp::min(
      start + self.first.unwrap_or(DEFAULT_PER_PAGE),
      self.before.unwrap_or(total),
    );
    if let Some(last) = self.last {
      start = if last > end - start { end } else { end - last };
    }

    let mut connection =
      async_graphql::connection::Connection::<usize, G>::new(start > 0, end < total);
    let scope = self
      .select
      .clone()
      .limit((end - start).try_into().unwrap())
      .offset(start.try_into().unwrap());

    connection.append(
      scope
        .all(db)
        .await?
        .into_iter()
        .enumerate()
        .map(|(index, model)| Edge::new(start + index, (iter)(model))),
    );

    Ok(connection)
  }
}

const DEFAULT_PER_PAGE: usize = 10;

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
          .relay_connection(db, |model| EventType { model }, after, before, first, last)
          .to_connection()
          .await?;

        Ok::<_, Error>(connection)
      },
    )
    .await
  }
}
