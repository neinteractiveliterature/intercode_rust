use std::sync::PoisonError;

use bumpalo_herd::Herd;
use futures::try_join;
use intercode_entities::events;
use intercode_liquid::liquid_datetime_to_chrono_datetime;
use liquid::{ObjectView, ValueView};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Select};
use seawater::DropResult;
use seawater::{Context, DropError, ModelBackedDrop};

use super::{drop_context::DropContext, EventDrop};

pub struct EventsCreatedSince {
  context: DropContext,
  convention_id: i64,
  herd: Herd,
}

impl std::fmt::Debug for EventsCreatedSince {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("EventsCreatedSince")
      .field("context", &self.context)
      .field("convention_id", &self.convention_id)
      .finish_non_exhaustive()
  }
}

impl Clone for EventsCreatedSince {
  fn clone(&self) -> Self {
    Self {
      context: self.context.clone(),
      convention_id: self.convention_id,
      herd: Default::default(),
    }
  }
}

impl EventsCreatedSince {
  pub fn new(convention_id: i64, context: DropContext) -> Self {
    EventsCreatedSince {
      context,
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
      scope.filter(events::Column::CreatedAt.gte(liquid_datetime_to_chrono_datetime(&start_date)))
    } else {
      scope
    }
  }

  async fn query_and_store(
    &self,
    start_date: Option<liquid::model::DateTime>,
  ) -> Result<&dyn ValueView, DropError> {
    let value = self
      .select_for_start_date(start_date)
      .all(self.context.db())
      .await?
      .into_iter()
      .map(|event| EventDrop::new(event, self.context.clone()))
      .collect::<Vec<_>>();

    let drops = self.context.with_drop_store(|drop_cache| {
      drop_cache
        .normalize_all(value)
        .map_err(|_| PoisonError::new(()))
    })?;

    try_join![
      EventDrop::preload_runs(self.context.clone(), &drops),
      EventDrop::preload_team_member_user_con_profiles(self.context.clone(), &drops),
      EventDrop::preload_event_category(self.context.clone(), &drops)
    ]?;

    let bump = self.herd.get();
    Ok(bump.alloc(drops))
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
    })
    .unwrap();

    Some(result)
  }
}

impl From<EventsCreatedSince> for DropResult<EventsCreatedSince> {
  fn from(value: EventsCreatedSince) -> Self {
    DropResult::new(value)
  }
}
