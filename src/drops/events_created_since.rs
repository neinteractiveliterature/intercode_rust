use intercode_entities::events;
use intercode_graphql::SchemaData;
use liquid::{ObjectView, ValueView};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Select};
use typed_arena::Arena;

use super::EventDrop;

static EMPTY_RESULT: liquid::model::Value = liquid::model::Value::Array(vec![]);

pub struct EventsCreatedSince {
  schema_data: SchemaData,
  convention_id: i64,
  arena: Arena<liquid::model::Value>,
}

impl std::fmt::Debug for EventsCreatedSince {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("EventsCreatedSince")
      .field("schema_data", &self.schema_data)
      .field("convention_id", &self.convention_id)
      .finish_non_exhaustive()
  }
}

impl Clone for EventsCreatedSince {
  fn clone(&self) -> Self {
    Self {
      schema_data: self.schema_data.clone(),
      convention_id: self.convention_id.clone(),
      arena: Default::default(),
    }
  }
}

impl EventsCreatedSince {
  pub fn new(schema_data: SchemaData, convention_id: i64) -> Self {
    EventsCreatedSince {
      schema_data,
      convention_id,
      arena: Default::default(),
    }
  }

  fn select_for_start_date(&self, start_date: liquid::model::DateTime) -> Select<events::Entity> {
    events::Entity::find()
      .filter(events::Column::ConventionId.eq(self.convention_id))
      .filter(events::Column::CreatedAt.gte(start_date.to_rfc2822()))
  }

  async fn query_and_store(&self, start_date: liquid::model::DateTime) -> &liquid::model::Value {
    let value = self
      .select_for_start_date(start_date)
      .all(self.schema_data.db.as_ref())
      .await
      .unwrap_or_else(|_| vec![])
      .into_iter()
      .map(EventDrop::new)
      .collect::<Vec<_>>()
      .to_value();

    self.arena.alloc(value)
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
      Some(start_date) => {
        let result = tokio::task::block_in_place(|| {
          tokio::runtime::Handle::current()
            .block_on(async move { self.query_and_store(start_date).await })
        });

        Some(result.as_view())
      }
      None => Some(EMPTY_RESULT.as_view()),
    }
  }
}
