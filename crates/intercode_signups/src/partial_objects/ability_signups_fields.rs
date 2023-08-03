use std::sync::Arc;

use async_graphql::*;
use intercode_entities::{conventions, events, runs, signups};
use intercode_graphql_core::{lax_id::LaxId, query_data::QueryData};
use intercode_graphql_loaders::LoaderManager;
use intercode_policies::{
  model_action_permitted::model_action_permitted, AuthorizationInfo, Policy,
};
use sea_orm::EntityTrait;
use seawater::loaders::ExpectModel;

use crate::policies::{SignupAction, SignupPolicy};

pub struct AbilitySignupsFields {
  authorization_info: Arc<AuthorizationInfo>,
}

impl AbilitySignupsFields {
  pub fn new(authorization_info: Arc<AuthorizationInfo>) -> Self {
    Self { authorization_info }
  }

  async fn get_signup_policy_model(
    &self,
    ctx: &Context<'_>,
    signup_id: ID,
  ) -> Result<
    (
      conventions::Model,
      events::Model,
      runs::Model,
      signups::Model,
    ),
    Error,
  > {
    let query_data = ctx.data::<QueryData>()?;
    let loaders = ctx.data::<Arc<LoaderManager>>()?;
    let signup = signups::Entity::find_by_id(LaxId::parse(signup_id)?)
      .one(query_data.db())
      .await?
      .ok_or_else(|| Error::new("Signup not found"))?;

    let run_result = loaders.signup_run().load_one(signup.id).await?;
    let run = run_result.expect_one()?;

    let event_result = loaders.run_event().load_one(run.id).await?;
    let event = event_result.expect_one()?;

    let convention_result = loaders.event_convention().load_one(event.id).await?;
    let convention = convention_result.expect_one()?;

    Ok((convention.clone(), event.clone(), run.clone(), signup))
  }
}

#[Object]
impl AbilitySignupsFields {
  #[graphql(name = "can_read_signups")]
  async fn can_read_signups(&self, ctx: &Context<'_>) -> Result<bool> {
    let convention = ctx.data::<QueryData>()?.convention();

    if let Some(convention) = convention {
      let event = events::Model {
        convention_id: convention.id,
        ..Default::default()
      };
      let run = runs::Model::default();
      let signup = signups::Model::default();

      model_action_permitted(
        &self.authorization_info,
        SignupPolicy,
        ctx,
        &SignupAction::Read,
        |_ctx| Ok(Some((convention.clone(), event, run, signup))),
      )
      .await
    } else {
      Ok(false)
    }
  }

  #[graphql(name = "can_manage_signups")]
  async fn can_manage_signups(&self, ctx: &Context<'_>) -> Result<bool> {
    let authorization_info = self.authorization_info.as_ref();
    let convention = ctx.data::<QueryData>()?.convention();
    let Some(convention) = convention else {
      return Ok(false);
    };

    Ok(
      SignupPolicy::action_permitted(
        authorization_info,
        &SignupAction::Manage,
        &(
          convention.clone(),
          events::Model {
            convention_id: convention.id,
            ..Default::default()
          },
          runs::Model::default(),
          signups::Model::default(),
        ),
      )
      .await?,
    )
  }

  #[graphql(name = "can_force_confirm_signup")]
  async fn can_force_confirm_signup(
    &self,
    ctx: &Context<'_>,
    signup_id: ID,
  ) -> Result<bool, Error> {
    let policy_model = self.get_signup_policy_model(ctx, signup_id).await?;

    model_action_permitted(
      self.authorization_info.as_ref(),
      SignupPolicy,
      ctx,
      &SignupAction::ForceConfirm,
      |_ctx| Ok(Some(&policy_model)),
    )
    .await
  }

  #[graphql(name = "can_update_bucket_signup")]
  async fn can_update_bucket_signup(
    &self,
    ctx: &Context<'_>,
    signup_id: ID,
  ) -> Result<bool, Error> {
    let policy_model = self.get_signup_policy_model(ctx, signup_id).await?;

    model_action_permitted(
      self.authorization_info.as_ref(),
      SignupPolicy,
      ctx,
      &SignupAction::UpdateBucket,
      |_ctx| Ok(Some(&policy_model)),
    )
    .await
  }

  #[graphql(name = "can_update_counted_signup")]
  async fn can_update_counted_signup(
    &self,
    ctx: &Context<'_>,
    signup_id: ID,
  ) -> Result<bool, Error> {
    let policy_model = self.get_signup_policy_model(ctx, signup_id).await?;

    model_action_permitted(
      self.authorization_info.as_ref(),
      SignupPolicy,
      ctx,
      &SignupAction::UpdateCounted,
      |_ctx| Ok(Some(&policy_model)),
    )
    .await
  }
}
