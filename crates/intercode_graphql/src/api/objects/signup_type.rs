use std::sync::Arc;

use async_graphql::{futures_util::try_join, *};
use chrono::{Datelike, NaiveDate};
use intercode_entities::{conventions, events, runs, signups};
use intercode_graphql_core::{enums::SignupState, policy_guard::PolicyGuard};
use intercode_graphql_loaders::LoaderManager;
use intercode_policies::policies::{SignupAction, SignupPolicy};
use seawater::loaders::ExpectModel;

use crate::{load_one_by_model_id, loader_result_to_required_single, model_backed_type};

use super::{RunType, UserConProfileType};

model_backed_type!(SignupType, signups::Model);

fn age_as_of(birth_date: NaiveDate, date: NaiveDate) -> i32 {
  let on_or_after_birthday = date.month() > birth_date.month()
    || (date.month() == birth_date.month() && date.day() >= birth_date.day());

  // subtract 1 year if they haven't yet reached their birthday on this date
  let subtract_years = i32::from(!on_or_after_birthday);

  date.year() - birth_date.year() - subtract_years
}

impl SignupType {
  fn policy_guard(
    &self,
    action: SignupAction,
  ) -> PolicyGuard<
    '_,
    SignupPolicy,
    (
      conventions::Model,
      events::Model,
      runs::Model,
      signups::Model,
    ),
    signups::Model,
  > {
    PolicyGuard::new(action, &self.model, move |model, ctx| {
      let model = model.clone();
      let ctx = ctx;
      let loaders = ctx.data::<Arc<LoaderManager>>();

      Box::pin(async {
        let loaders = loaders?;
        let signup_run_loader = loaders.signup_run();
        let run_event_loader = loaders.run_event();
        let event_convention_loader = loaders.event_convention();
        let run_result = signup_run_loader.load_one(model.id).await?;
        let run = run_result.expect_one()?;
        let event_result = run_event_loader.load_one(run.id).await?;
        let event = event_result.expect_one()?;
        let convention_result = event_convention_loader.load_one(event.id).await?;
        let convention = convention_result.expect_one()?;

        Ok((convention.clone(), event.clone(), run.clone(), model))
      })
    })
  }
}

#[Object(name = "Signup", guard = "self.policy_guard(SignupAction::Read)")]
impl SignupType {
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
        if age_as_of(birth_date, run.starts_at.unwrap().date()) >= minimum_age {
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

  async fn run(&self, ctx: &Context<'_>) -> Result<RunType, Error> {
    let loader_result = load_one_by_model_id!(signup_run, ctx, self)?;
    Ok(loader_result_to_required_single!(loader_result, RunType))
  }

  async fn state(&self) -> Result<SignupState> {
    SignupState::try_from(self.model.state.as_str())
  }

  #[graphql(name = "user_con_profile")]
  async fn user_con_profile(&self, ctx: &Context<'_>) -> Result<UserConProfileType, Error> {
    let loader_result = load_one_by_model_id!(signup_user_con_profile, ctx, self)?;
    Ok(loader_result_to_required_single!(
      loader_result,
      UserConProfileType
    ))
  }

  #[graphql(name = "waitlist_position")]
  async fn waitlist_position(&self, ctx: &Context<'_>) -> Result<Option<usize>, Error> {
    Ok(
      ctx
        .data::<Arc<LoaderManager>>()?
        .signup_waitlist_position
        .load_one(self.model.clone().into())
        .await?
        .flatten(),
    )
  }
}
