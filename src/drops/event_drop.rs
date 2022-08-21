use intercode_entities::{events, runs};
use intercode_graphql::SchemaData;
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
use liquid::model::DateTime;
use sea_orm::ModelTrait;

use super::{utils::naive_date_time_to_liquid_date_time, DropError, RunDrop};

#[liquid_drop_struct]
pub struct EventDrop {
  event: events::Model,
  schema_data: SchemaData,
}

#[liquid_drop_impl]
impl EventDrop {
  pub fn new(event: events::Model, schema_data: SchemaData) -> Self {
    EventDrop { event, schema_data }
  }

  fn id(&self) -> i64 {
    self.event.id
  }

  fn created_at(&self) -> Option<DateTime> {
    self
      .event
      .created_at
      .and_then(naive_date_time_to_liquid_date_time)
  }

  async fn runs(&self) -> Result<Vec<RunDrop>, DropError> {
    Ok(
      self
        .event
        .find_related(runs::Entity)
        .all(self.schema_data.db.as_ref())
        .await?
        .into_iter()
        .map(RunDrop::new)
        .collect::<Vec<_>>(),
    )
  }
}
