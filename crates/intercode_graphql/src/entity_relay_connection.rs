use async_graphql::connection::Edge;
use sea_orm::{ConnectionTrait, EntityTrait, FromQueryResult, PaginatorTrait, QuerySelect, Select};

pub const DEFAULT_PER_PAGE: usize = 10;
pub const MAX_PAGE_SIZE: usize = 100;

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
    self,
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
  pub async fn total_count(&self) -> Result<usize, sea_orm::DbErr> {
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
  ) -> Result<async_graphql::connection::Connection<usize, G>, sea_orm::DbErr> {
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
