use async_graphql::dataloader::Loader;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use intercode_entities::{model_ext::time_bounds::TimeBoundsSelectExt, runs};
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use seawater::ConnectionWrapper;
use std::{collections::HashMap, sync::Arc};

pub struct FilteredEventRunsLoader {
  db: ConnectionWrapper,
  filter: EventRunsLoaderFilter,
}

impl FilteredEventRunsLoader {
  pub fn new(db: ConnectionWrapper, filter: EventRunsLoaderFilter) -> Self {
    FilteredEventRunsLoader { db, filter }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct EventRunsLoaderFilter {
  pub start: Option<NaiveDateTime>,
  pub finish: Option<NaiveDateTime>,
}

#[async_trait]
impl Loader<i64> for FilteredEventRunsLoader {
  type Value = Vec<runs::Model>;
  type Error = Arc<DbErr>;

  async fn load(
    &self,
    keys: &[i64],
  ) -> Result<std::collections::HashMap<i64, Self::Value>, Self::Error> {
    Ok(
      runs::Entity::find()
        .filter(runs::Column::EventId.is_in(keys.iter().copied()))
        .between(self.filter.start, self.filter.finish)
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
