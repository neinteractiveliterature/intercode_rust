use bumpalo_herd::Herd;
use futures::join;
use intercode_entities::events;
use intercode_graphql::SchemaData;
use lazy_liquid_value_view::DropResult;
use liquid::{ObjectView, ValueView};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Select};
use seawater::{preloaders::Preloader, ModelBackedDrop};

use crate::drops::UserConProfileDrop;

use super::EventDrop;

pub struct EventsCreatedSince {
  schema_data: SchemaData,
  convention_id: i64,
  herd: Herd,
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
      convention_id: self.convention_id,
      herd: Default::default(),
    }
  }
}

impl EventsCreatedSince {
  pub fn new(schema_data: SchemaData, convention_id: i64) -> Self {
    EventsCreatedSince {
      schema_data,
      convention_id,
      herd: Default::default(),
    }
  }

  fn select_for_start_date(
    &self,
    start_date: Option<liquid::model::DateTime>,
  ) -> Select<events::Entity> {
    let scope = events::Entity::find().filter(events::Column::ConventionId.eq(self.convention_id));

    if let Some(start_date) = start_date {
      scope.filter(events::Column::CreatedAt.gte(start_date.to_rfc2822()))
    } else {
      scope
    }
  }

  async fn query_and_store(&self, start_date: Option<liquid::model::DateTime>) -> &dyn ValueView {
    let value = self
      .select_for_start_date(start_date)
      .all(self.schema_data.db.as_ref())
      .await
      .unwrap_or_else(|_| vec![])
      .into_iter()
      .map(|event| EventDrop::new(event, self.schema_data.clone()))
      .collect::<Vec<_>>();

    join![
      async {
        EventDrop::runs_preloader()
          .preload(
            &self.schema_data.db,
            value.iter().collect::<Vec<_>>().as_slice(),
          )
          .await
          .ok()
      },
      async {
        let result = EventDrop::team_member_user_con_profiles_preloader()
          .preload(
            &self.schema_data.db,
            value.iter().collect::<Vec<_>>().as_slice(),
          )
          .await;

        if let Ok(result) = result {
          let values = result.all_values_flat_unwrapped();

          UserConProfileDrop::preload_users_and_signups(
            self.schema_data.clone(),
            values.iter().collect::<Vec<_>>().as_slice(),
          )
          .await
          .ok();
        }
      },
      async {
        EventDrop::event_category_preloader()
          .preload(
            &self.schema_data.db,
            value.iter().collect::<Vec<_>>().as_slice(),
          )
          .await
          .ok()
      }
    ];

    let bump = self.herd.get();
    bump.alloc(value)
  }
}

impl ValueView for EventsCreatedSince {
  fn as_debug(&self) -> &dyn std::fmt::Debug {
    self
  }

  fn as_object(&self) -> Option<&dyn ObjectView> {
    Some(self)
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
    unimplemented!()
  }
}

impl ObjectView for EventsCreatedSince {
  fn as_value(&self) -> &dyn ValueView {
    self
  }

  fn size(&self) -> i64 {
    unimplemented!()
  }

  fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = liquid::model::KStringCow<'k>> + 'k> {
    unimplemented!()
  }

  fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn ValueView> + 'k> {
    unimplemented!()
  }

  fn iter<'k>(
    &'k self,
  ) -> Box<dyn Iterator<Item = (liquid::model::KStringCow<'k>, &'k dyn ValueView)> + 'k> {
    unimplemented!()
  }

  fn contains_key(&self, _index: &str) -> bool {
    true
  }

  fn get<'s>(&'s self, index: &str) -> Option<&'s dyn ValueView> {
    let start_date = liquid::model::DateTime::from_str(index);

    let result = tokio::task::block_in_place(|| {
      tokio::runtime::Handle::current()
        .block_on(async move { self.query_and_store(start_date).await })
    });

    Some(result)
  }
}

impl From<EventsCreatedSince> for DropResult<EventsCreatedSince> {
  fn from(value: EventsCreatedSince) -> Self {
    DropResult::new(value)
  }
}
