use intercode_entities::runs;
use liquid::model::DateTime;
use seawater::{belongs_to_related, has_many_related, model_backed_drop, DropError};
use seawater::{liquid_drop_impl, liquid_drop_struct};
use time::Duration;

use super::{
  drop_context::DropContext, utils::naive_date_time_to_liquid_date_time, EventDrop, RoomDrop,
};

model_backed_drop!(RunDrop, runs::Model, DropContext);

#[belongs_to_related(event, EventDrop, eager_load(event_category))]
#[has_many_related(rooms, RoomDrop, serialize = true)]
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
    if let Some(starts_at) = self.starts_at().await.get_inner() {
      let mut starts_at = *starts_at;
      let event_length = self
        .event()
        .await
        .expect_inner()
        .length_seconds()
        .await
        .expect_inner();
      *starts_at += Duration::seconds((*event_length).into());
      Ok(Some(starts_at))
    } else {
      Ok(None)
    }
  }
}
