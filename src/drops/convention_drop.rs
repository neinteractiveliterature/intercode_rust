use chrono::Utc;
use intercode_entities::{conventions, MaximumEventSignupsValue};
use intercode_timespan::ScheduledValue;
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
use sea_orm::JsonValue;
use seawater::{has_many_related, model_backed_drop};

use super::{
  drop_context::DropContext, utils::naive_date_time_to_liquid_date_time, EventCategoryDrop,
  EventsCreatedSince, ScheduledValueDrop, StaffPositionDrop, StaffPositionsByName,
};

model_backed_drop!(ConventionDrop, conventions::Model, DropContext);

#[has_many_related(event_categories, EventCategoryDrop, serialize = true)]
#[has_many_related(staff_positions, StaffPositionDrop, serialize = true)]
#[liquid_drop_impl(i64)]
impl ConventionDrop {
  fn id(&self) -> i64 {
    self.model.id
  }

  fn name(&self) -> Option<&str> {
    self.model.name.as_deref()
  }

  fn events_created_since(&self) -> EventsCreatedSince {
    EventsCreatedSince::new(self.model.id, self.context.clone())
  }

  fn location(&self) -> Option<&JsonValue> {
    self.model.location.as_ref()
  }

  fn maximum_event_signups(&self) -> ScheduledValueDrop<Utc, MaximumEventSignupsValue> {
    self
      .model
      .maximum_event_signups
      .as_ref()
      .map(|maximum_event_signups| {
        let scheduled_value: ScheduledValue<Utc, MaximumEventSignupsValue> =
          serde_json::from_value(maximum_event_signups.clone()).unwrap_or_default();
        ScheduledValueDrop::new(scheduled_value, self.context.clone())
      })
      .unwrap_or_else(|| ScheduledValueDrop::new(Default::default(), self.context.clone()))
  }

  fn show_schedule(&self) -> &str {
    &self.model.show_schedule
  }

  fn staff_positions_by_name(&self) -> StaffPositionsByName {
    StaffPositionsByName::new(self.model.clone(), self.context.clone())
  }

  fn starts_at(&self) -> Option<liquid::model::DateTime> {
    self
      .model
      .starts_at
      .and_then(naive_date_time_to_liquid_date_time)
  }

  fn ends_at(&self) -> Option<liquid::model::DateTime> {
    self
      .model
      .ends_at
      .and_then(naive_date_time_to_liquid_date_time)
  }

  fn ticket_name(&self) -> &str {
    self.model.ticket_name.as_str()
  }
}
