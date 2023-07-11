use async_graphql::{connection::Edge, OutputType};
use sea_orm::{ConnectionTrait, EntityTrait, FromQueryResult, PaginatorTrait, QuerySelect, Select};

pub const DEFAULT_PER_PAGE: u64 = 10;
pub const MAX_PAGE_SIZE: u64 = 100;

pub trait RelayConnectable<
  'db,
  E: EntityTrait<Model = M>,
  M: FromQueryResult + Sized + Send + Sync + 'db,
  G,
  F: Fn(M) -> G,
>
{
  fn relay_connection<C: ConnectionTrait>(
    self,
    db: &'db C,
    to_graphql_representation: F,
    after: Option<u64>,
    before: Option<u64>,
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
    self,
    db: &'db C,
    to_graphql_representation: F,
    after: Option<u64>,
    before: Option<u64>,
    first: Option<usize>,
    last: Option<usize>,
  ) -> RelayConnectionWrapper<'db, C, M, E, G, F> {
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
  after: Option<u64>,
  before: Option<u64>,
  first: Option<u64>,
  last: Option<u64>,
}

impl<'db, C, M, E, G, F> RelayConnectionWrapper<'db, C, M, E, G, F>
where
  C: ConnectionTrait,
  M: FromQueryResult + Sized + Send + Sync + 'db,
  E: EntityTrait<Model = M>,
  F: Fn(M) -> G + Copy,
  G: OutputType,
{
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
  ) -> Result<async_graphql::connection::Connection<u64, G>, sea_orm::DbErr> {
    let iter = self.to_graphql_representation;
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

    let mut connection =
      async_graphql::connection::Connection::<u64, G>::new(start > 0, end < total);
    let scope = self.select.clone().limit(end - start).offset(start);

    connection.edges.extend(
      scope
        .all(db)
        .await?
        .into_iter()
        .enumerate()
        .map(|(index, model)| Edge::new(start + u64::try_from(index).unwrap(), (iter)(model))),
    );

    Ok(connection)
  }
}
