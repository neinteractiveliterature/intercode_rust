use intercode_entities::runs;
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
use liquid::model::DateTime;

use super::utils::naive_date_time_to_liquid_date_time;

#[liquid_drop_struct]
pub struct RunDrop {
  run: runs::Model,
}

#[liquid_drop_impl]
impl RunDrop {
  pub fn new(run: runs::Model) -> Self {
    RunDrop { run }
  }

  fn id(&self) -> i64 {
    self.run.id
  }

  fn created_at(&self) -> Option<DateTime> {
    self
      .run
      .created_at
      .and_then(naive_date_time_to_liquid_date_time)
  }

  fn starts_at(&self) -> Option<DateTime> {
    self
      .run
      .starts_at
      .and_then(naive_date_time_to_liquid_date_time)
  }
}
