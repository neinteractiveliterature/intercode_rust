use intercode_entities::{user_con_profiles, users};
use oxide_auth::endpoint::Scope;
use sea_orm::TransactionTrait;
use seawater::ConnectionWrapper;
use std::{future::Future, pin::Pin, sync::Arc};

use crate::AuthorizationInfo;

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

impl AuthorizationInfo {
  pub async fn for_test(
    db: ConnectionWrapper,
    user: Option<users::Model>,
    oauth_scope: Option<Scope>,
    assumed_identity_from_profile: Option<user_con_profiles::Model>,
  ) -> Self {
    Self::new(db, user, oauth_scope, assumed_identity_from_profile)
  }
}
