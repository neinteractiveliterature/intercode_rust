use axum::{async_trait, response::Response};
use axum_sessions::{
  async_session::{MemoryStore, Session, SessionStore},
  SessionLayer,
};
use chrono::Utc;
use futures::future::BoxFuture;
use http::Request;
use intercode_entities::sessions;
use sea_orm::{sea_query::OnConflict, ColumnTrait, EntityTrait, QueryFilter};
use seawater::ConnectionWrapper;
use tower::{Layer, Service};
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

#[async_trait]
impl SessionStore for DbSessionStore {
  async fn load_session(
    &self,
    cookie_value: String,
  ) -> axum_sessions::async_session::Result<Option<Session>> {
    let session_id = Session::id_from_cookie_value(&cookie_value)?;

    sessions::Entity::find()
      .filter(sessions::Column::SessionId.eq(session_id.clone()))
      .one(self.db.as_ref())
      .await
      .map(|find_result| {
        find_result
          .and_then(|record| record.data)
          .and_then(|encoded| base64::decode(encoded).ok())
          .and_then(|bytes| String::from_utf8(bytes).ok())
          .and_then(|data| serde_json::from_str::<Session>(&data).ok())
      })
      .map_err(|err| err.into())
  }

  async fn store_session(
    &self,
    session: Session,
  ) -> axum_sessions::async_session::Result<Option<String>> {
    let session_id = session.id().to_string();
    let encoded_data = base64::encode(serde_json::to_string(&session)?);
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
    Ok(session.into_cookie_value())
  }

  async fn destroy_session(
    &self,
    session: axum_sessions::async_session::Session,
  ) -> axum_sessions::async_session::Result {
    sessions::Entity::delete_many()
      .filter(sessions::Column::SessionId.eq(session.id()))
      .exec(self.db.as_ref())
      .await?;

    Ok(())
  }

  async fn clear_store(&self) -> axum_sessions::async_session::Result {
    sessions::Entity::delete_many()
      .exec(self.db.as_ref())
      .await?;

    Ok(())
  }
}

#[derive(Clone)]
pub struct SessionWithDbStoreFromTxLayer {
  secret: [u8; 64],
}

impl SessionWithDbStoreFromTxLayer {
  pub fn new(secret: [u8; 64]) -> Self {
    Self { secret }
  }
}

impl<S> Layer<S> for SessionWithDbStoreFromTxLayer {
  type Service = SessionWithDbStoreFromTxService<S>;

  fn layer(&self, inner: S) -> Self::Service {
    SessionWithDbStoreFromTxService {
      secret: self.secret,
      inner,
    }
  }
}

#[derive(Clone)]
pub struct SessionWithDbStoreFromTxService<S> {
  secret: [u8; 64],
  inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for SessionWithDbStoreFromTxService<S>
where
  S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
  ResBody: Send + 'static,
  ReqBody: Send + 'static,
  S::Future: Send + 'static,
{
  type Response = Response<ResBody>;
  type Error = S::Error;
  type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

  fn poll_ready(
    &mut self,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Result<(), Self::Error>> {
    self.inner.poll_ready(cx)
  }

  fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
    let inner = self.inner.clone();
    let secret = self.secret;
    Box::pin(async move {
      let (parts, body) = req.into_parts();
      let db = parts.extensions.get::<ConnectionWrapper>();

      match db {
        Some(wrapper) => {
          let store = DbSessionStore::new(wrapper.clone());
          let layer = SessionLayer::new(store, &secret);
          let mut service = layer.layer(inner);
          let req = Request::from_parts(parts, body);
          service.call(req).await
        }
        None => {
          error!("Couldn't get ConnectionWrapper from request extensions");
          let store = MemoryStore::new();
          let layer = SessionLayer::new(store, &secret);
          let mut service = layer.layer(inner);
          let req = Request::from_parts(parts, body);
          service.call(req).await
        }
      }
    })
  }
}
