use async_graphql::{async_trait, dataloader::Loader};
use intercode_entities::form_response_changes;
use sea_orm::{ColumnTrait, DbErr, QueryFilter, Select};
use seawater::ConnectionWrapper;
use std::{collections::HashMap, sync::Arc};

pub struct FormResponseChangesLoader {
  db: ConnectionWrapper,
  scope: Select<form_response_changes::Entity>,
}

impl FormResponseChangesLoader {
  pub fn new(db: ConnectionWrapper, scope: Select<form_response_changes::Entity>) -> Self {
    FormResponseChangesLoader { db, scope }
  }
}

#[async_trait::async_trait]
impl Loader<i64> for FormResponseChangesLoader {
  type Value = Vec<form_response_changes::Model>;
  type Error = Arc<DbErr>;

  async fn load(
    &self,
    keys: &[i64],
  ) -> Result<std::collections::HashMap<i64, Self::Value>, Self::Error> {
    Ok(
      self
        .scope
        .clone()
        .filter(form_response_changes::Column::ResponseId.is_in(keys.iter().copied()))
        .all(&self.db)
        .await?
        .into_iter()
        .fold(
          HashMap::<i64, Self::Value>::with_capacity(keys.len()),
          |mut acc, change| {
            if let Some(response_id) = change.response_id {
              let changes = acc.entry(response_id).or_default();
              changes.push(change);
            }
            acc
          },
        ),
    )
  }
}
