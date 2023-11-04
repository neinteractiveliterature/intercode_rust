use std::{error::Error, fmt::Display};

use axum::{async_trait, BoxError};
use chrono::Utc;
use futures::future::BoxFuture;
use http::Request;
use intercode_entities::sessions;
use sea_orm::{sea_query::OnConflict, ColumnTrait, DbErr, EntityTrait, QueryFilter};
use seawater::ConnectionWrapper;
use tower::{Layer, Service};
use tower_sessions::{
  session::SessionId, MemoryStore, Session, SessionManager, SessionManagerLayer, SessionRecord,
  SessionStore,
};
use tracing::log::error;

#[derive(Clone, Debug)]
pub struct DbSessionStore {
  db: ConnectionWrapper,
}

impl DbSessionStore {
  pub fn new(db: ConnectionWrapper) -> Self {
    DbSessionStore { db }
  }
}

#[derive(Debug)]
pub enum DbSessionError {
  DbErr(DbErr),
  SerializationError(serde_json::Error),
}

impl Display for DbSessionError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      DbSessionError::DbErr(err) => err.fmt(f),
      DbSessionError::SerializationError(err) => err.fmt(f),
    }
  }
}

impl Error for DbSessionError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    None
  }

  fn description(&self) -> &str {
    "description() is deprecated; use Display"
  }

  fn cause(&self) -> Option<&dyn Error> {
    self.source()
  }
}

impl From<DbErr> for DbSessionError {
  fn from(value: DbErr) -> Self {
    Self::DbErr(value)
  }
}

impl From<serde_json::Error> for DbSessionError {
  fn from(value: serde_json::Error) -> Self {
    Self::SerializationError(value)
  }
}

#[async_trait]
impl SessionStore for DbSessionStore {
  type Error = DbSessionError;

  async fn save(&self, session_record: &SessionRecord) -> Result<(), Self::Error> {
    let session_id = session_record.id().to_string();
    let encoded_data = serde_json::to_string(&session_record)?;
    let model = sessions::ActiveModel {
      id: sea_orm::ActiveValue::NotSet,
      created_at: sea_orm::ActiveValue::Set(Some(Utc::now().naive_utc())),
      updated_at: sea_orm::ActiveValue::Set(Some(Utc::now().naive_utc())),
      session_id: sea_orm::ActiveValue::Set(session_id),
      data: sea_orm::ActiveValue::Set(Some(encoded_data)),
    };
    sessions::Entity::insert(model)
      .on_conflict(
        OnConflict::column(sessions::Column::SessionId)
          .update_columns(vec![sessions::Column::UpdatedAt, sessions::Column::Data].into_iter())
          .to_owned(),
      )
      .exec(self.db.as_ref())
      .await?;
    Ok(())
  }

  async fn load(&self, session_id: &SessionId) -> Result<Option<Session>, Self::Error> {
    sessions::Entity::find()
      .filter(sessions::Column::SessionId.eq(session_id.0.to_string()))
      .one(self.db.as_ref())
      .await
      .map(|find_result| {
        find_result
          .and_then(|record| record.data)
          .and_then(|data| serde_json::from_str::<SessionRecord>(&data).ok())
          .map(Session::from)
      })
      .map_err(DbSessionError::from)
  }

  async fn delete(&self, session_id: &SessionId) -> Result<(), Self::Error> {
    sessions::Entity::delete_many()
      .filter(sessions::Column::SessionId.eq(session_id.0.to_string()))
      .exec(self.db.as_ref())
      .await?;

    Ok(())
  }
}

#[derive(Clone)]
pub struct SessionWithDbStoreFromTxLayer;

impl SessionWithDbStoreFromTxLayer {
  pub fn new() -> Self {
    Self {}
  }
}

impl<S> Layer<S> for SessionWithDbStoreFromTxLayer {
  type Service = SessionWithDbStoreFromTxService<S>;

  fn layer(&self, inner: S) -> Self::Service {
    SessionWithDbStoreFromTxService { inner }
  }
}

#[derive(Clone)]
pub struct SessionWithDbStoreFromTxService<S> {
  inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for SessionWithDbStoreFromTxService<S>
where
  S: Service<Request<ReqBody>, Response = http::Response<ResBody>> + Clone + Send + 'static,
  ResBody: Send + 'static,
  ReqBody: Send + 'static,
  S::Future: Send + 'static,
  S::Error: Error + Send + Sync,
{
  type Response = <SessionManager<S, DbSessionStore> as Service<Request<ReqBody>>>::Response;
  type Error = BoxError;
  type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

  fn poll_ready(
    &mut self,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Result<(), Self::Error>> {
    self
      .inner
      .poll_ready(cx)
      .map_err(|err| Box::new(err) as BoxError)
  }

  fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
    let inner = self.inner.clone();
    Box::pin(async move {
      let (parts, body) = req.into_parts();
      let db = parts.extensions.get::<ConnectionWrapper>();

      match db {
        Some(wrapper) => {
          let store = DbSessionStore::new(wrapper.clone());
          let layer = SessionManagerLayer::new(store);
          let mut service = layer.layer(inner);
          let req = Request::from_parts(parts, body);
          service.call(req).await
        }
        None => {
          error!("Couldn't get ConnectionWrapper from request extensions");
          let store = MemoryStore::default();
          let layer = SessionManagerLayer::new(store);
          let mut service = layer.layer(inner);
          let req = Request::from_parts(parts, body);
          service.call(req).await
        }
      }
    })
  }
}
