use async_graphql::{
  dataloader::{DataLoader, Loader},
  futures_util::lock::Mutex,
};
use async_session::async_trait;
use chrono::NaiveDateTime;
use intercode_entities::runs;
use sea_orm::{sea_query::Expr, ColumnTrait, DbErr, EntityTrait, QueryFilter};
use seawater::ConnectionWrapper;
use std::{collections::HashMap, sync::Arc, time::Duration};

pub struct EventRunsLoader {
  db: ConnectionWrapper,
  filter: EventRunsLoaderFilter,
}

impl EventRunsLoader {
  pub fn new(db: ConnectionWrapper, filter: EventRunsLoaderFilter) -> Self {
    EventRunsLoader { db, filter }
  }
}

#[derive(Debug)]
pub struct EventRunsLoaderManager {
  loaders_by_filter: Arc<Mutex<HashMap<EventRunsLoaderFilter, Arc<DataLoader<EventRunsLoader>>>>>,
  db: ConnectionWrapper,
  delay: Duration,
}

impl EventRunsLoaderManager {
  pub fn new(db: ConnectionWrapper, delay: Duration) -> Self {
    EventRunsLoaderManager {
      loaders_by_filter: Default::default(),
      db,
      delay,
    }
  }

  pub async fn with_filter(
    &self,
    filter: EventRunsLoaderFilter,
  ) -> Arc<DataLoader<EventRunsLoader>> {
    let mut lock = self.loaders_by_filter.lock().await;
    let loader = lock.entry(filter.clone()).or_insert_with(|| {
      Arc::new(
        DataLoader::new(EventRunsLoader::new(self.db.clone(), filter), tokio::spawn)
          .delay(self.delay),
      )
    });

    loader.clone()
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct EventRunsLoaderFilter {
  pub start: Option<NaiveDateTime>,
  pub finish: Option<NaiveDateTime>,
}

#[async_trait]
impl Loader<i64> for EventRunsLoader {
  type Value = Vec<runs::Model>;
  type Error = Arc<DbErr>;

  async fn load(
    &self,
    keys: &[i64],
  ) -> Result<std::collections::HashMap<i64, Self::Value>, Self::Error> {
    Ok(
      runs::Entity::find()
        .filter(runs::Column::EventId.is_in(keys.iter().copied()))
        .filter(Expr::cust_with_values(
          "tsrange(?, ?, '[)') && timespan_tsrange",
          vec![self.filter.start, self.filter.finish],
        ))
        .all(&self.db)
        .await?
        .into_iter()
        .fold(
          HashMap::<i64, Self::Value>::with_capacity(keys.len()),
          |mut acc, run| {
            let runs = acc.entry(run.event_id).or_insert_with(Default::default);
            runs.push(run);
            acc
          },
        ),
    )
  }
}
