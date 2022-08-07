use std::sync::Arc;

use chrono::Utc;
use i18n_embed::fluent::FluentLanguageLoader;
use intercode_entities::{conventions, MaximumEventSignupsValue};
use intercode_graphql::{loaders::expect::ExpectModels, SchemaData};
use intercode_timespan::ScheduledValue;
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
use sea_orm::JsonValue;

use super::{
  utils::naive_date_time_to_liquid_date_time, DropError, EventCategoryDrop, EventsCreatedSince,
  ScheduledValueDrop,
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

  async fn event_categories(&self) -> Result<Vec<EventCategoryDrop<'cache>>, DropError> {
    let result = self
      .schema_data
      .loaders
      .convention_event_categories
      .load_one(self.convention.id)
      .await?;
    let models = result.expect_models()?;

    Ok(
      models
        .iter()
        .map(|event_category| EventCategoryDrop::new(event_category.clone()))
        .collect::<Vec<_>>(),
    )
  }

  fn events_created_since(&self) -> &EventsCreatedSince {
    &self.events_created_since
  }

  fn location(&self) -> Option<&JsonValue> {
    self.convention.location.as_ref()
  }

  fn maximum_event_signups(&self) -> ScheduledValueDrop<MaximumEventSignupsValue> {
    self
      .convention
      .maximum_event_signups
      .as_ref()
      .map(|maximum_event_signups| {
        let scheduled_value: ScheduledValue<Utc, MaximumEventSignupsValue> =
          serde_json::from_value(maximum_event_signups.clone()).unwrap_or_default();
        ScheduledValueDrop::new(scheduled_value, self.language_loader.as_ref())
      })
      .unwrap_or_else(|| {
        ScheduledValueDrop::new::<Utc>(Default::default(), self.language_loader.as_ref())
      })
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
