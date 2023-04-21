use async_graphql::{futures_util::try_join, *};
use chrono::{Datelike, NaiveDate};
use intercode_entities::{events, runs, signups};
use intercode_policies::policies::{SignupAction, SignupPolicy};
use seawater::loaders::ExpectModels;

use crate::{api::enums::SignupState, model_backed_type, policy_guard::PolicyGuard, QueryData};

use super::{ModelBackedType, RunType, UserConProfileType};

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
  ) -> PolicyGuard<'_, SignupPolicy, (events::Model, runs::Model, signups::Model), signups::Model>
  {
    PolicyGuard::new(action, &self.model, move |model, ctx| {
      let model = model.clone();
      let ctx = ctx;
      let query_data = ctx.data::<QueryData>();

      Box::pin(async {
        let query_data = query_data?;
        let signup_run_loader = query_data.loaders().signup_run();
        let run_event_loader = query_data.loaders().run_event();
        let run_result = signup_run_loader.load_one(model.id).await?;
        let run = run_result.expect_one()?;
        let event_result = run_event_loader.load_one(run.id).await?;
        let event = event_result.expect_one()?;

        Ok((event.clone(), run.clone(), model))
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
    let query_data = ctx.data::<QueryData>()?;

    let (user_con_profile, (run, event)) = try_join!(
      async {
        Ok::<_, Error>(
          query_data
            .loaders()
            .signup_user_con_profile()
            .load_one(self.model.id)
            .await?
            .expect_one()?
            .clone(),
        )
      },
      async {
        let run = query_data
          .loaders()
          .signup_run()
          .load_one(self.model.id)
          .await?
          .expect_one()?
          .clone();
        let event = query_data
          .loaders()
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
    let query_data = ctx.data::<QueryData>()?;

    Ok(RunType::new(
      query_data
        .loaders()
        .signup_run()
        .load_one(self.model.id)
        .await?
        .expect_one()?
        .clone(),
    ))
  }

  async fn state(&self) -> Result<SignupState> {
    SignupState::try_from(self.model.state.as_str())
  }

  #[graphql(name = "user_con_profile")]
  async fn user_con_profile(&self, ctx: &Context<'_>) -> Result<UserConProfileType, Error> {
    let query_data = ctx.data::<QueryData>()?;

    Ok(UserConProfileType::new(
      query_data
        .loaders()
        .signup_user_con_profile()
        .load_one(self.model.id)
        .await?
        .expect_one()?
        .clone(),
    ))
  }

  #[graphql(name = "waitlist_position")]
  async fn waitlist_position(&self, ctx: &Context<'_>) -> Result<Option<usize>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    Ok(
      query_data
        .loaders()
        .signup_waitlist_position
        .load_one(self.model.clone().into())
        .await?
        .flatten(),
    )
  }
}
