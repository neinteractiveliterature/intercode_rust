use intercode_entities::{links::SignupToEvent, signups};
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
use liquid::model::DateTime;
use seawater::{belongs_to_linked, belongs_to_related, model_backed_drop, DropError};

use super::{drop_context::DropContext, EventDrop, RunDrop};

model_backed_drop!(SignupDrop, signups::Model, DropContext);

#[belongs_to_related(run, RunDrop, serialize = true)]
#[belongs_to_linked(event, EventDrop, SignupToEvent, serialize = true)]
#[liquid_drop_impl(i64)]
impl SignupDrop {
  fn id(&self) -> i64 {
    self.model.id
  }

  fn team_member(&self) -> bool {
    // TODO
    false
  }

  async fn ends_at(&self) -> Result<Option<DateTime>, DropError> {
    let run = self.run().await?;
    eprintln!("{:?}", run);
    let ends_at = run.ends_at().await;
    eprintln!("{:?}", ends_at);
    ends_at
  }

  fn state(&self) -> &str {
    &self.model.state
  }

  async fn starts_at(&self) -> Result<Option<DateTime>, DropError> {
    Ok(self.run().await?.starts_at())
  }
}
