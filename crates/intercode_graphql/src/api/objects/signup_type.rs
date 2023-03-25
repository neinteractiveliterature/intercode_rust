use async_graphql::{futures_util::try_join, *};
use chrono::{Datelike, NaiveDate};
use intercode_entities::signups;
use seawater::loaders::ExpectModels;

use crate::{model_backed_type, QueryData};

use super::{ModelBackedType, RunType, UserConProfileType};

model_backed_type!(SignupType, signups::Model);

fn age_as_of(birth_date: NaiveDate, date: NaiveDate) -> i32 {
  let on_or_after_birthday = date.month() > birth_date.month()
    || (date.month() == birth_date.month() && date.day() >= birth_date.day());

  // subtract 1 year if they haven't yet reached their birthday on this date
  let subtract_years = i32::from(!on_or_after_birthday);

  date.year() - birth_date.year() - subtract_years
}

#[Object(name = "Signup")]
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

  async fn state(&self) -> &str {
    &self.model.state
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
