use std::sync::Arc;

use crate::policies::{SignupAction, SignupPolicy};
use async_graphql::{futures_util::try_join, *};
use chrono::{Datelike, NaiveDate};
use intercode_entities::{runs, signups, user_con_profiles};
use intercode_graphql_core::{
  enums::SignupState, load_one_by_model_id, model_backed_type, ModelBackedType,
};
use intercode_graphql_loaders::LoaderManager;
use intercode_policies::ModelBackedTypeGuardablePolicy;
use seawater::loaders::ExpectModel;

fn age_as_of(birth_date: NaiveDate, date: NaiveDate) -> i32 {
  let on_or_after_birthday = date.month() > birth_date.month()
    || (date.month() == birth_date.month() && date.day() >= birth_date.day());

  // subtract 1 year if they haven't yet reached their birthday on this date
  let subtract_years = i32::from(!on_or_after_birthday);

  date.year() - birth_date.year() - subtract_years
}

model_backed_type!(SignupSignupsFields, signups::Model);

impl SignupSignupsFields {
  pub async fn run(&self, ctx: &Context<'_>) -> Result<runs::Model, Error> {
    let loader_result = load_one_by_model_id!(signup_run, ctx, self)?;
    Ok(loader_result.expect_one()?.clone())
  }

  pub async fn user_con_profile(
    &self,
    ctx: &Context<'_>,
  ) -> Result<user_con_profiles::Model, Error> {
    let loader_result = load_one_by_model_id!(signup_user_con_profile, ctx, self)?;
    Ok(loader_result.expect_one()?.clone())
  }
}

#[Object(guard = "SignupPolicy::model_guard(SignupAction::Read, self)")]
impl SignupSignupsFields {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "age_restrictions_check")]
  async fn age_restrictions_check(&self, ctx: &Context<'_>) -> Result<&str, Error> {
    let loaders = ctx.data::<Arc<LoaderManager>>()?;

    let (user_con_profile, (run, event)) = try_join!(
      async {
        Ok::<_, Error>(
          loaders
            .signup_user_con_profile()
            .load_one(self.model.id)
            .await?
            .expect_one()?
            .clone(),
        )
      },
      async {
        let run = loaders
          .signup_run()
          .load_one(self.model.id)
          .await?
          .expect_one()?
          .clone();
        let event = loaders
          .run_event()
          .load_one(run.id)
          .await?
          .expect_one()?
          .clone();
        Ok::<_, Error>((run, event))
      }
    )?;

    if let Some(minimum_age) = event.minimum_age {
      if let Some(birth_date) = user_con_profile.birth_date {
        if age_as_of(birth_date, run.starts_at.date()) >= minimum_age {
          Ok("OK")
        } else {
          Ok("Too young")
        }
      } else {
        Ok("Unknown age")
      }
    } else {
      Ok("N/A")
    }
  }

  #[graphql(name = "bucket_key")]
  async fn bucket_key(&self) -> Option<&str> {
    self.model.bucket_key.as_deref()
  }

  async fn counted(&self) -> bool {
    self.model.counted.unwrap_or(false)
  }

  #[graphql(name = "requested_bucket_key")]
  async fn requested_bucket_key(&self) -> Option<&str> {
    self.model.requested_bucket_key.as_deref()
  }

  async fn state(&self) -> Result<SignupState> {
    SignupState::try_from(self.model.state.as_str())
  }

  #[graphql(name = "waitlist_position")]
  async fn waitlist_position(&self, ctx: &Context<'_>) -> Result<Option<usize>, Error> {
    Ok(
      ctx
        .data::<Arc<LoaderManager>>()?
        .signup_waitlist_position
        .load_one(self.get_model().into())
        .await?
        .flatten(),
    )
  }
}
