use std::future::Future;

use async_graphql::{
  connection::{query, Connection, Edge},
  Error, OutputType,
};
use sea_orm::{
  ConnectionTrait, EntityTrait, FromQueryResult, ModelTrait, PaginatorTrait, QuerySelect, Select,
};
use seawater::ConnectionWrapper;

use crate::ModelBackedType;

pub const DEFAULT_PER_PAGE: u64 = 10;
pub const MAX_PAGE_SIZE: u64 = 100;

pub trait RelayConnectable<'db, M: ModelTrait + Sized + Send + Sync + 'db, G> {
  fn relay_connection<C: ConnectionTrait>(
    self,
    db: &'db C,
    to_graphql_representation: Box<dyn Fn(M) -> G + Send + Sync>,
    after: Option<u64>,
    before: Option<u64>,
    first: Option<usize>,
    last: Option<usize>,
  ) -> RelayConnectionWrapper<'db, C, M, G>;
}

impl<'db, M: ModelTrait + Sized + Send + Sync + 'db, G> RelayConnectable<'db, M, G>
  for Select<M::Entity>
{
  fn relay_connection<C: ConnectionTrait>(
    self,
    db: &'db C,
    to_graphql_representation: Box<dyn Fn(M) -> G + Send + Sync>,
    after: Option<u64>,
    before: Option<u64>,
    first: Option<usize>,
    last: Option<usize>,
  ) -> RelayConnectionWrapper<'db, C, M, G> {
    RelayConnectionWrapper {
      select: self,
      db,
      to_graphql_representation,
      after,
      before,
      first: first.map(|v| v.try_into().unwrap()),
      last: last.map(|v| v.try_into().unwrap()),
    }
  }
}

pub struct RelayConnectionWrapper<'db, C, M, G>
where
  C: ConnectionTrait,
  M: ModelTrait + Sized + Send + Sync + 'db,
{
  select: Select<M::Entity>,
  db: &'db C,
  to_graphql_representation: Box<dyn Fn(M) -> G + Send + Sync>,
  after: Option<u64>,
  before: Option<u64>,
  first: Option<u64>,
  last: Option<u64>,
}

impl<'db, C, M, G> RelayConnectionWrapper<'db, C, M, G>
where
  C: ConnectionTrait,
  M: ModelTrait + FromQueryResult + Sized + Send + Sync + 'db,
  G: OutputType,
{
  pub fn into_type<O: ModelBackedType<Model = M>>(self) -> RelayConnectionWrapper<'db, C, M, O>
  where
    G: ModelBackedType<Model = M> + 'static,
    M: 'static,
  {
    RelayConnectionWrapper {
      select: self.select,
      db: self.db,
      to_graphql_representation: Box::new(move |m| (self.to_graphql_representation)(m).into_type()),
      after: self.after,
      before: self.before,
      first: self.first,
      last: self.last,
    }
  }

  pub fn from_type<O: ModelBackedType<Model = M> + Send + Sync + OutputType>(
    conn: RelayConnectionWrapper<'db, C, M, O>,
  ) -> Self
  where
    G: ModelBackedType<Model = M> + 'static,
    O: 'static,
    M: 'static,
  {
    conn.into_type()
  }

  pub async fn total_count(&self) -> Result<u64, sea_orm::DbErr> {
    self
      .select
      .clone()
      .into_model::<M>()
      .paginate(self.db, 1)
      .num_items()
      .await
  }

  pub async fn to_connection(
    &self,
  ) -> Result<async_graphql::connection::Connection<u64, G>, sea_orm::DbErr>
  where
    <<M as ModelTrait>::Entity as EntityTrait>::Model: Into<M>,
  {
    let db = self.db;

    let total = self.total_count().await?;

    let mut start = self.after.map(|after| after + 1).unwrap_or(0);
    let mut end = std::cmp::min(
      start + self.first.unwrap_or(DEFAULT_PER_PAGE),
      self.before.unwrap_or(total),
    );
    if let Some(last) = self.last {
      start = if last > end - start { end } else { end - last };
    }
    if end - start > MAX_PAGE_SIZE {
      end = start + MAX_PAGE_SIZE;
    }

    let scope = self.select.clone().limit(end - start).offset(start);

    let mut connection =
      async_graphql::connection::Connection::<u64, G>::new(start > 0, end < total);

    connection.edges.extend(
      scope
        .all(db)
        .await?
        .into_iter()
        .enumerate()
        .map(|(index, model)| {
          Edge::new(
            start + u64::try_from(index).unwrap(),
            (self.to_graphql_representation)(model.into()),
          )
        }),
    );

    Ok(connection)
  }
}

pub async fn type_converting_query<
  'a,
  A: ModelBackedType + OutputType + 'static,
  B: ModelBackedType<Model = A::Model> + OutputType,
  F: FnOnce(Option<u64>, Option<u64>, Option<usize>, Option<usize>) -> R,
  R: Future<Output = Result<RelayConnectionWrapper<'a, ConnectionWrapper, A::Model, A>, Error>>,
>(
  after: Option<String>,
  before: Option<String>,
  first: Option<i32>,
  last: Option<i32>,
  f: F,
) -> Result<Connection<u64, B>, Error>
where
  A::Model: FromQueryResult
    + Sync
    + 'a
    + From<<<<A as ModelBackedType>::Model as ModelTrait>::Entity as EntityTrait>::Model>,
{
  query(
    after,
    before,
    first,
    last,
    |after, before, first, last| async move {
      Ok::<Connection<u64, B>, Error>(
        (f)(after, before, first, last)
          .await
          .map(|res| res.into_type())?
          .to_connection()
          .await?,
      )
    },
  )
  .await
}
