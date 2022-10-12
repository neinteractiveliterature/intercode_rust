use axum::async_trait;
use axum_sessions::async_session::{Session, SessionStore};
use chrono::Utc;
use intercode_entities::sessions;
use sea_orm::{sea_query::OnConflict, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct DbSessionStore {
  db: Arc<DatabaseConnection>,
}

impl DbSessionStore {
  pub fn new(db: Arc<DatabaseConnection>) -> Self {
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
