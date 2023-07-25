mod authorization_info;
pub mod model_action_permitted;
mod permissions_loading;
pub mod policies;
mod policy;
mod policy_guard;
mod simple_policy_guard;

use async_graphql::{Context, Error};
pub use authorization_info::*;
use intercode_graphql_core::{ModelBackedType, ModelPaginator};
use intercode_query_builders::{PaginationFromQueryBuilder, QueryBuilder};
pub use permissions_loading::*;
pub use policy::*;
pub use policy_guard::*;
use sea_orm::{FromQueryResult, ModelTrait, Select};
pub use simple_policy_guard::*;

pub trait AuthorizedFromQueryBuilder<Model: ModelTrait>: PaginationFromQueryBuilder<Model>
where
  Model: Sync,
{
  fn authorized_from_query_builder<
    B: QueryBuilder<Entity = Model::Entity>,
    P: EntityPolicy<AuthorizationInfo, Model, Action = A>,
    A: From<ReadManageAction>,
  >(
    query_builder: &B,
    ctx: &Context<'_>,
    scope: Select<Model::Entity>,
    page: Option<u64>,
    per_page: Option<u64>,
    _policy: P,
  ) -> Result<Self, Error>
  where
    Self: Sized,
  {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let scope = P::filter_scope(scope, authorization_info, &A::from(ReadManageAction::Read));
    Ok(Self::from_query_builder(
      query_builder,
      scope,
      page,
      per_page,
    ))
  }
}

impl<Item: ModelBackedType> AuthorizedFromQueryBuilder<Item::Model> for ModelPaginator<Item> where
  Item::Model: Sync + FromQueryResult
{
}

#[cfg(test)]
mod test_helpers {
  use sea_orm::TransactionTrait;
  use seawater::ConnectionWrapper;
  use std::{future::Future, pin::Pin, sync::Arc};

  pub async fn with_test_db<F>(f: F)
  where
    F: FnOnce(ConnectionWrapper) -> Pin<Box<dyn Future<Output = ()>>> + 'static,
  {
    use sea_orm::{ConnectOptions, Database};

    let db = Database::connect(ConnectOptions::new(
      "postgres://postgres@localhost/intercode_test".to_string(),
    ))
    .await
    .unwrap();

    let tx = Arc::new(db.begin().await.unwrap());
    f(ConnectionWrapper::DatabaseTransaction(Arc::downgrade(&tx))).await;

    let tx = Arc::try_unwrap(tx).unwrap();
    tx.rollback().await.unwrap();
  }
}
