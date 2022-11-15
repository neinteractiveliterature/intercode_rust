use async_trait::async_trait;
use axum_sea_orm_tx::Tx;
use futures::lock::Mutex;
use sea_orm::{
  ConnectionTrait, DatabaseBackend, DatabaseConnection, DatabaseTransaction, TransactionError,
  TransactionTrait,
};
use std::{fmt::Debug, future::Future, pin::Pin, sync::Arc};

#[derive(Clone, Debug)]
pub enum ConnectionWrapper {
  DatabaseConnection(Arc<DatabaseConnection>),
  DatabaseTransaction(Arc<Mutex<DatabaseTransaction>>, DatabaseBackend),
  Tx(Arc<Mutex<Tx<ConnectionWrapper>>>, DatabaseBackend),
}

impl AsRef<Self> for ConnectionWrapper {
  fn as_ref(&self) -> &Self {
    self
  }
}

impl From<DatabaseConnection> for ConnectionWrapper {
  fn from(conn: DatabaseConnection) -> Self {
    Self::DatabaseConnection(Arc::new(conn))
  }
}

impl From<DatabaseTransaction> for ConnectionWrapper {
  fn from(tx: DatabaseTransaction) -> Self {
    let backend = tx.get_database_backend();
    Self::DatabaseTransaction(Arc::new(Mutex::new(tx)), backend)
  }
}

impl From<Tx<ConnectionWrapper>> for ConnectionWrapper {
  fn from(tx: Tx<ConnectionWrapper>) -> Self {
    let backend = tx.get_database_backend();
    Self::Tx(Arc::new(Mutex::new(tx)), backend)
  }
}

impl From<Arc<DatabaseConnection>> for ConnectionWrapper {
  fn from(arc: Arc<DatabaseConnection>) -> Self {
    Self::DatabaseConnection(arc)
  }
}

impl From<Arc<Mutex<DatabaseTransaction>>> for ConnectionWrapper {
  fn from(arc: Arc<Mutex<DatabaseTransaction>>) -> Self {
    let backend = arc.try_lock().unwrap().get_database_backend();
    Self::DatabaseTransaction(arc, backend)
  }
}

impl From<Arc<Mutex<Tx<ConnectionWrapper>>>> for ConnectionWrapper {
  fn from(arc: Arc<Mutex<Tx<ConnectionWrapper>>>) -> Self {
    let backend = arc.try_lock().unwrap().get_database_backend();
    Self::Tx(arc, backend)
  }
}

impl From<&ConnectionWrapper> for ConnectionWrapper {
  fn from(wrapper: &ConnectionWrapper) -> Self {
    wrapper.clone()
  }
}

#[async_trait]
impl ConnectionTrait for ConnectionWrapper {
  fn get_database_backend(&self) -> sea_orm::DbBackend {
    match self {
      Self::DatabaseConnection(conn) => conn.get_database_backend(),
      Self::DatabaseTransaction(_tx, backend) => *backend,
      Self::Tx(_tx, backend) => *backend,
    }
  }

  async fn execute(&self, stmt: sea_orm::Statement) -> Result<sea_orm::ExecResult, sea_orm::DbErr> {
    match self {
      Self::DatabaseConnection(conn) => conn.execute(stmt).await,
      Self::DatabaseTransaction(tx, _backend) => tx.lock().await.execute(stmt).await,
      Self::Tx(tx, _backend) => tx.lock().await.execute(stmt).await,
    }
  }

  async fn query_one(
    &self,
    stmt: sea_orm::Statement,
  ) -> Result<Option<sea_orm::QueryResult>, sea_orm::DbErr> {
    match self {
      Self::DatabaseConnection(conn) => conn.query_one(stmt).await,
      Self::DatabaseTransaction(tx, _backend) => tx.lock().await.query_one(stmt).await,
      Self::Tx(tx, _backend) => tx.lock().await.query_one(stmt).await,
    }
  }

  async fn query_all(
    &self,
    stmt: sea_orm::Statement,
  ) -> Result<Vec<sea_orm::QueryResult>, sea_orm::DbErr> {
    match self {
      Self::DatabaseConnection(conn) => conn.query_all(stmt).await,
      Self::DatabaseTransaction(tx, _backend) => tx.lock().await.query_all(stmt).await,
      Self::Tx(tx, _backend) => tx.lock().await.query_all(stmt).await,
    }
  }
}

#[async_trait]
impl TransactionTrait for ConnectionWrapper {
  async fn begin(&self) -> Result<DatabaseTransaction, sea_orm::DbErr> {
    match self {
      Self::DatabaseConnection(conn) => conn.begin().await,
      Self::DatabaseTransaction(tx, _backend) => tx.lock().await.begin().await,
      Self::Tx(tx, _backend) => tx.lock().await.begin().await,
    }
  }

  async fn transaction<F, T, E>(&self, callback: F) -> Result<T, TransactionError<E>>
  where
    F: for<'c> FnOnce(
        &'c DatabaseTransaction,
      ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>
      + Send,
    T: Send,
    E: std::error::Error + Send,
  {
    match self {
      Self::DatabaseConnection(conn) => conn.transaction(callback).await,
      Self::DatabaseTransaction(tx, _backend) => tx.lock().await.transaction(callback).await,
      Self::Tx(tx, _backend) => tx.lock().await.transaction(callback).await,
    }
  }
}
