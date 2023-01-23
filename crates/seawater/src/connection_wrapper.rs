use async_trait::async_trait;
use sea_orm::{
  AccessMode, ConnectionTrait, DatabaseConnection, DatabaseTransaction, IsolationLevel,
  TransactionError, TransactionTrait,
};
use std::{
  fmt::Debug,
  future::Future,
  pin::Pin,
  sync::{Arc, Weak},
};

#[derive(Clone, Debug)]
pub enum ConnectionWrapper {
  DatabaseConnection(Arc<DatabaseConnection>),
  DatabaseTransaction(Weak<DatabaseTransaction>),
}

impl Drop for ConnectionWrapper {
  fn drop(&mut self) {
    // eprintln!("Dropping ConnectionWrapper {:?}", self);
  }
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

impl From<Arc<DatabaseConnection>> for ConnectionWrapper {
  fn from(arc: Arc<DatabaseConnection>) -> Self {
    Self::DatabaseConnection(arc)
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
      Self::DatabaseTransaction(tx) => tx.upgrade().unwrap().get_database_backend(),
    }
  }

  async fn execute(&self, stmt: sea_orm::Statement) -> Result<sea_orm::ExecResult, sea_orm::DbErr> {
    match self {
      Self::DatabaseConnection(conn) => conn.execute(stmt).await,
      Self::DatabaseTransaction(tx) => {
        tx.upgrade()
          .ok_or_else(|| sea_orm::DbErr::Custom("Transaction has already ended".to_string()))?
          .execute(stmt)
          .await
      }
    }
  }

  async fn query_one(
    &self,
    stmt: sea_orm::Statement,
  ) -> Result<Option<sea_orm::QueryResult>, sea_orm::DbErr> {
    match self {
      Self::DatabaseConnection(conn) => conn.query_one(stmt).await,
      Self::DatabaseTransaction(tx) => {
        tx.upgrade()
          .ok_or_else(|| sea_orm::DbErr::Custom("Transaction has already ended".to_string()))?
          .query_one(stmt)
          .await
      }
    }
  }

  async fn query_all(
    &self,
    stmt: sea_orm::Statement,
  ) -> Result<Vec<sea_orm::QueryResult>, sea_orm::DbErr> {
    match self {
      Self::DatabaseConnection(conn) => conn.query_all(stmt).await,
      Self::DatabaseTransaction(tx) => {
        tx.upgrade()
          .ok_or_else(|| sea_orm::DbErr::Custom("Transaction has already ended".to_string()))?
          .query_all(stmt)
          .await
      }
    }
  }
}

#[async_trait]
impl TransactionTrait for ConnectionWrapper {
  async fn begin(&self) -> Result<DatabaseTransaction, sea_orm::DbErr> {
    match self {
      Self::DatabaseConnection(conn) => conn.begin().await,
      Self::DatabaseTransaction(tx) => {
        tx.upgrade()
          .ok_or_else(|| sea_orm::DbErr::Custom("Transaction has already ended".to_string()))?
          .begin()
          .await
      }
    }
  }

  async fn begin_with_config(
    &self,
    isolation_level: Option<IsolationLevel>,
    access_mode: Option<AccessMode>,
  ) -> Result<DatabaseTransaction, sea_orm::DbErr> {
    match self {
      Self::DatabaseConnection(conn) => conn.begin_with_config(isolation_level, access_mode).await,
      Self::DatabaseTransaction(tx) => {
        tx.upgrade()
          .ok_or_else(|| sea_orm::DbErr::Custom("Transaction has already ended".to_string()))?
          .begin_with_config(isolation_level, access_mode)
          .await
      }
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
      Self::DatabaseTransaction(tx) => {
        let upgraded = tx.upgrade();
        match upgraded {
          None => Err(TransactionError::Connection(sea_orm::DbErr::Custom(
            "Transaction has already ended".to_string(),
          ))),
          Some(tx) => tx.transaction(callback).await,
        }
      }
    }
  }

  async fn transaction_with_config<F, T, E>(
    &self,
    callback: F,
    isolation_level: Option<IsolationLevel>,
    access_mode: Option<AccessMode>,
  ) -> Result<T, TransactionError<E>>
  where
    F: for<'c> FnOnce(
        &'c DatabaseTransaction,
      ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>
      + Send,
    T: Send,
    E: std::error::Error + Send,
  {
    match self {
      Self::DatabaseConnection(conn) => {
        conn
          .transaction_with_config(callback, isolation_level, access_mode)
          .await
      }
      Self::DatabaseTransaction(tx) => {
        let upgraded = tx.upgrade();
        match upgraded {
          None => Err(TransactionError::Connection(sea_orm::DbErr::Custom(
            "Transaction has already ended".to_string(),
          ))),
          Some(tx) => {
            tx.transaction_with_config(callback, isolation_level, access_mode)
              .await
          }
        }
      }
    }
  }
}
