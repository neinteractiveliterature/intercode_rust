use intercode_entities::runs;
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
use liquid::model::DateTime;
use seawater::model_backed_drop;

use super::utils::naive_date_time_to_liquid_date_time;

model_backed_drop!(RunDrop, runs::Model);

#[liquid_drop_impl]
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

  fn starts_at(&self) -> Option<DateTime> {
    self
      .model
      .starts_at
      .and_then(naive_date_time_to_liquid_date_time)
  }
}
