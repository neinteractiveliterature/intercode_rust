mod authorization_info;
pub mod model_action_permitted;
mod permissions_loading;
pub mod policies;
mod policy;
mod policy_guard;
mod simple_policy_guard;

pub use authorization_info::*;
pub use permissions_loading::*;
pub use policy::*;
pub use policy_guard::*;
pub use simple_policy_guard::*;

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
