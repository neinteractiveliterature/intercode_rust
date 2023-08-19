use intercode_entities::links::{
  SignupRequestToEvent, SignupRequestToReplaceSignup, SignupRequestToResultSignup,
};
use intercode_entities::signup_requests;
use seawater::{belongs_to_linked, belongs_to_related, model_backed_drop};
use seawater::{has_one_linked, liquid_drop_impl};

use super::{drop_context::DropContext, RunDrop, UserConProfileDrop};
use super::{EventDrop, SignupDrop};

model_backed_drop!(SignupRequestDrop, signup_requests::Model, DropContext);

#[belongs_to_related(target_run, RunDrop, serialize = true, eager_load(event))]
#[belongs_to_linked(event, EventDrop, SignupRequestToEvent)]
#[has_one_linked(replace_signup, SignupDrop, SignupRequestToReplaceSignup)]
#[has_one_linked(result_signup, SignupDrop, SignupRequestToResultSignup)]
#[belongs_to_related(user_con_profile, UserConProfileDrop)]
#[liquid_drop_impl(i64, DropContext)]
impl SignupRequestDrop {
  fn id(&self) -> i64 {
    self.model.id
  }
}
