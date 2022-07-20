use std::{
  collections::HashMap,
  sync::{Arc, RwLock},
};

use intercode_entities::events;
use intercode_graphql::SchemaData;
use liquid::{ObjectView, ValueView};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Select};
use tokio::sync::OnceCell;

use super::EventDrop;

static EMPTY_RESULT: liquid::model::Value = liquid::model::Value::Array(vec![]);

#[derive(Debug, Clone)]
pub struct EventsCreatedSince {
  schema_data: SchemaData,
  convention_id: i64,
  result_cache: Arc<RwLock<HashMap<liquid::model::DateTime, OnceCell<liquid::model::Value>>>>,
  sequenced_results: Arc<RwLock<Vec<liquid::model::Value>>>,
}

impl EventsCreatedSince {
  pub fn new(schema_data: SchemaData, convention_id: i64) -> Self {
    EventsCreatedSince {
      schema_data,
      convention_id,
      result_cache: Default::default(),
      sequenced_results: Default::default(),
    }
  }

  fn select_for_start_date(&self, start_date: liquid::model::DateTime) -> Select<events::Entity> {
    events::Entity::find()
      .filter(events::Column::ConventionId.eq(self.convention_id))
      .filter(events::Column::CreatedAt.gte(start_date.to_rfc2822()))
  }

  // async fn get_or_init_for_date_sequenced(
  //   &self,
  //   start_date: liquid::model::DateTime,
  // ) -> Option<&liquid::model::Value> {
  //   let result = self
  //     .select_for_start_date(start_date)
  //     .all(self.schema_data.db.as_ref())
  //     .await;

  //   let result = result.ok().map(|result| {
  //     result
  //       .into_iter()
  //       .map(|event| EventDrop::new(event))
  //       .collect::<Vec<_>>()
  //       .to_value()
  //   });

  //   if let Some(result) = result {
  //     let mut sequence = self.sequenced_results.write().unwrap();
  //     sequence.push(result);
  //     drop(sequence);

  //     let sequence = self.sequenced_results.read().unwrap();
  //     let last_item = sequence.last();
  //     last_item.map(|value| &*value)
  //   } else {
  //     None
  //   }
  // }

  async fn get_or_init_for_date(
    &self,
    start_date: liquid::model::DateTime,
  ) -> Option<&dyn liquid::ValueView> {
    let mut cache = self.result_cache.write().unwrap();
    let entry = cache.entry(start_date);
    entry.or_insert(Default::default());
    drop(cache);

    let cache = &*(self.result_cache.read().unwrap());
    let cell = cache.get(&start_date).unwrap();
    let result = cell
      .get_or_try_init(|| async {
        Ok::<_, sea_orm::DbErr>(
          self
            .select_for_start_date(start_date)
            .all(self.schema_data.db.as_ref())
            .await?
            .into_iter()
            .map(|event| EventDrop::new(event))
            .collect::<Vec<_>>()
            .to_value(),
        )
      })
      .await
      .ok();

    result.map(|value| value.as_view())
  }
}

impl ValueView for EventsCreatedSince {
  fn as_debug(&self) -> &dyn std::fmt::Debug {
    self
  }

  fn render(&self) -> liquid::model::DisplayCow<'_> {
    liquid::model::DisplayCow::Owned(Box::new("EventsCreatedSince"))
  }

  fn source(&self) -> liquid::model::DisplayCow<'_> {
    liquid::model::DisplayCow::Owned(Box::new("EventsCreatedSince"))
  }

  fn type_name(&self) -> &'static str {
    "EventsCreatedSince"
  }

  fn query_state(&self, state: liquid::model::State) -> bool {
    match state {
      liquid::model::State::Truthy => true,
      liquid::model::State::DefaultValue => false,
      liquid::model::State::Empty => false,
      liquid::model::State::Blank => false,
    }
  }

  fn to_kstr(&self) -> liquid::model::KStringCow<'_> {
    "EventsCreatedSince".to_kstr()
  }

  fn to_value(&self) -> liquid_core::Value {
    todo!()
  }
}

impl ObjectView for EventsCreatedSince {
  fn as_value(&self) -> &dyn ValueView {
    self
  }

  fn size(&self) -> i64 {
    todo!()
  }

  fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = liquid::model::KStringCow<'k>> + 'k> {
    todo!()
  }

  fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn ValueView> + 'k> {
    todo!()
  }

  fn iter<'k>(
    &'k self,
  ) -> Box<dyn Iterator<Item = (liquid::model::KStringCow<'k>, &'k dyn ValueView)> + 'k> {
    todo!()
  }

  fn contains_key(&self, index: &str) -> bool {
    liquid::model::DateTime::from_str(index).is_some()
  }

  fn get<'s>(&'s self, index: &str) -> Option<&'s dyn ValueView> {
    let start_date = liquid::model::DateTime::from_str(index);

    match start_date {
      Some(start_date) => tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
          .block_on(async move { self.get_or_init_for_date(start_date).await })
      })
      .map(|arc| arc.as_ref()),
      None => Some(EMPTY_RESULT.as_view()),
    }
  }
}
