use intercode_entities::runs;
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
use liquid::model::DateTime;
use seawater::{belongs_to_related, model_backed_drop, DropError};
use time::Duration;

use super::{drop_context::DropContext, utils::naive_date_time_to_liquid_date_time, EventDrop};

model_backed_drop!(RunDrop, runs::Model, DropContext);

#[belongs_to_related(event, EventDrop)]
#[liquid_drop_impl(i64)]
impl RunDrop {
  fn id(&self) -> i64 {
    self.model.id
  }

  fn created_at(&self) -> Option<DateTime> {
    self
      .model
      .created_at
      .and_then(naive_date_time_to_liquid_date_time)
  }

  pub fn starts_at(&self) -> Option<DateTime> {
    self
      .model
      .starts_at
      .and_then(naive_date_time_to_liquid_date_time)
  }

  pub async fn ends_at(&self) -> Result<Option<DateTime>, DropError> {
    if let Some(mut starts_at) = self.starts_at() {
      let event_length = self.event().await?.length_seconds();
      *starts_at += Duration::seconds(event_length.into());
      Ok(Some(starts_at))
    } else {
      Ok(None)
    }
  }
}
