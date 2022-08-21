use std::sync::Arc;

use chrono::Utc;
use i18n_embed::fluent::FluentLanguageLoader;
use intercode_entities::{
  conventions, event_categories, links::ConventionToStaffPositions, MaximumEventSignupsValue,
};
use intercode_graphql::SchemaData;
use intercode_timespan::ScheduledValue;
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
use sea_orm::{JsonValue, ModelTrait};

use super::{
  utils::naive_date_time_to_liquid_date_time, DropError, EventCategoryDrop, EventsCreatedSince,
  ScheduledValueDrop, StaffPositionDrop, StaffPositionsByName,
};

#[liquid_drop_struct]
pub struct ConventionDrop {
  schema_data: SchemaData,
  convention: conventions::Model,
  events_created_since: EventsCreatedSince,
  language_loader: Arc<FluentLanguageLoader>,
}

#[liquid_drop_impl]
impl ConventionDrop {
  pub fn new(
    schema_data: SchemaData,
    convention: conventions::Model,
    language_loader: Arc<FluentLanguageLoader>,
  ) -> Self {
    let convention_id = convention.id;

    ConventionDrop {
      schema_data: schema_data.clone(),
      convention,
      language_loader,
      events_created_since: EventsCreatedSince::new(schema_data, convention_id),
    }
  }

  fn id(&self) -> i64 {
    self.convention.id
  }

  fn name(&self) -> Option<&str> {
    self.convention.name.as_deref()
  }

  async fn event_categories(&self) -> Result<Vec<EventCategoryDrop>, DropError> {
    let models = self
      .convention
      .find_related(event_categories::Entity)
      .all(self.schema_data.db.as_ref())
      .await?;

    Ok(
      models
        .into_iter()
        .map(EventCategoryDrop::new)
        .collect::<Vec<_>>(),
    )
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

  async fn staff_positions(&self) -> Result<Vec<StaffPositionDrop>, DropError> {
    let drops = self
      .convention
      .find_linked(ConventionToStaffPositions)
      .all(self.schema_data.db.as_ref())
      .await?
      .into_iter()
      .filter(|staff_position| staff_position.visible.unwrap_or(false))
      .map(|staff_position| StaffPositionDrop::new(staff_position, self.schema_data.clone()))
      .collect::<Vec<_>>();

    StaffPositionDrop::preload_user_con_profiles(
      &self.schema_data,
      &drops.iter().collect::<Vec<_>>(),
    )
    .await?;

    Ok(drops)
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
