mod authorization_info;
mod form_response_policy;
mod permissions_loading;
pub mod policies;
mod policy;

pub use authorization_info::*;
pub use form_response_policy::*;
pub use permissions_loading::*;
pub use policy::*;

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
    f(tx.clone().into()).await;

    let tx = Arc::try_unwrap(tx).unwrap();
    tx.rollback().await.unwrap();
  }
}
