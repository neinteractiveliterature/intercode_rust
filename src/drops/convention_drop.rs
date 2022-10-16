use std::sync::Arc;

use chrono::Utc;
use i18n_embed::fluent::FluentLanguageLoader;
use intercode_entities::{conventions, MaximumEventSignupsValue};
use intercode_graphql::SchemaData;
use intercode_timespan::ScheduledValue;
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
use sea_orm::JsonValue;
use seawater::{has_many_related, ModelBackedDrop};

use super::{
  utils::naive_date_time_to_liquid_date_time, EventCategoryDrop, EventsCreatedSince,
  ScheduledValueDrop, StaffPositionDrop, StaffPositionsByName,
};

#[liquid_drop_struct]
pub struct ConventionDrop {
  schema_data: SchemaData,
  convention: conventions::Model,
  events_created_since: EventsCreatedSince,
  language_loader: Arc<FluentLanguageLoader>,
}

impl ModelBackedDrop for ConventionDrop {
  type Model = conventions::Model;

  fn new(model: Self::Model, schema_data: SchemaData) -> Self {
    let convention_id = model.id;

    ConventionDrop {
      schema_data: schema_data.clone(),
      convention: model,
      language_loader: schema_data.language_loader.clone(),
      events_created_since: EventsCreatedSince::new(schema_data, convention_id),
      drop_cache: Default::default(),
    }
  }

  fn get_model(&self) -> &Self::Model {
    &self.convention
  }
}

#[has_many_related(event_categories, EventCategoryDrop)]
#[has_many_related(staff_positions, StaffPositionDrop)]
#[liquid_drop_impl]
impl ConventionDrop {
  pub fn new(convention: conventions::Model, schema_data: SchemaData) -> Self {
    let convention_id = convention.id;

    ConventionDrop {
      schema_data: schema_data.clone(),
      convention,
      language_loader: schema_data.language_loader.clone(),
      events_created_since: EventsCreatedSince::new(schema_data, convention_id),
    }
  }

  fn id(&self) -> i64 {
    self.convention.id
  }

  fn name(&self) -> Option<&str> {
    self.convention.name.as_deref()
  }

  fn events_created_since(&self) -> &EventsCreatedSince {
    &self.events_created_since
  }

  fn location(&self) -> Option<&JsonValue> {
    self.convention.location.as_ref()
  }

  fn maximum_event_signups(&self) -> ScheduledValueDrop<Utc, MaximumEventSignupsValue> {
    self
      .convention
      .maximum_event_signups
      .as_ref()
      .map(|maximum_event_signups| {
        let scheduled_value: ScheduledValue<Utc, MaximumEventSignupsValue> =
          serde_json::from_value(maximum_event_signups.clone()).unwrap_or_default();
        ScheduledValueDrop::new(scheduled_value, self.language_loader.clone())
      })
      .unwrap_or_else(|| ScheduledValueDrop::new(Default::default(), self.language_loader.clone()))
  }

  fn show_schedule(&self) -> &str {
    &self.convention.show_schedule
  }

  fn staff_positions_by_name(&self) -> StaffPositionsByName {
    StaffPositionsByName::new(self.schema_data.clone(), self.convention.clone())
  }

  fn starts_at(&self) -> Option<liquid::model::DateTime> {
    self
      .convention
      .starts_at
      .and_then(naive_date_time_to_liquid_date_time)
  }

  fn ends_at(&self) -> Option<liquid::model::DateTime> {
    self
      .convention
      .ends_at
      .and_then(naive_date_time_to_liquid_date_time)
  }

  fn ticket_name(&self) -> &str {
    self.convention.ticket_name.as_str()
  }
}
