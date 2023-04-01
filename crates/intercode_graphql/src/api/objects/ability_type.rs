use std::borrow::Borrow;

use async_graphql::*;
use intercode_entities::{
  conventions, events, rooms, root_sites, runs, signups, user_con_profiles,
};
use intercode_policies::{
  policies::{
    CmsContentPolicy, ConventionAction, ConventionPolicy, EventAction, EventPolicy, RoomPolicy,
    SignupAction, SignupPolicy, UserConProfileAction, UserConProfilePolicy,
  },
  AuthorizationInfo, Policy, ReadManageAction,
};
use sea_orm::EntityTrait;
use seawater::loaders::ExpectModels;

use crate::{lax_id::LaxId, QueryData};

pub struct AbilityType;

async fn model_action_permitted<
  'a,
  P: Policy<AuthorizationInfo, M>,
  M: Send + Sync + 'a,
  R: Borrow<M>,
>(
  _policy: P,
  ctx: &'a Context<'_>,
  action: &P::Action,
  get_model: impl FnOnce(&'a Context<'_>) -> Result<Option<R>, Error>,
) -> Result<bool, Error> {
  let authorization_info = ctx.data::<AuthorizationInfo>()?;
  let model_ref = get_model(ctx)?;

  if let Some(model_ref) = model_ref {
    Ok(P::action_permitted(authorization_info, action, model_ref.borrow()).await?)
  } else {
    Ok(false)
  }
}

// TODO all the methods stubbed out with just "false"
#[Object(name = "Ability")]
impl AbilityType {
  #[graphql(name = "can_manage_conventions")]
  async fn can_manage_conventions(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    model_action_permitted(ConventionPolicy, ctx, &ConventionAction::Update, |_ctx| {
      Ok(Some(conventions::Model::default()))
    })
    .await
  }

  #[graphql(name = "can_read_schedule")]
  async fn can_read_schedule(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    model_action_permitted(ConventionPolicy, ctx, &ConventionAction::Schedule, |ctx| {
      Ok(ctx.data::<QueryData>()?.convention())
    })
    .await
  }

  #[graphql(name = "can_read_schedule_with_counts")]
  async fn can_read_schedule_with_counts(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    model_action_permitted(
      ConventionPolicy,
      ctx,
      &ConventionAction::ScheduleWithCounts,
      |ctx| Ok(ctx.data::<QueryData>()?.convention()),
    )
    .await
  }

  #[graphql(name = "can_list_events")]
  async fn can_list_events(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    model_action_permitted(
      ConventionPolicy,
      ctx,
      &ConventionAction::ListEvents,
      |ctx| Ok(ctx.data::<QueryData>()?.convention()),
    )
    .await
  }

  #[graphql(name = "can_read_user_con_profiles")]
  async fn can_read_user_con_profiles(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    model_action_permitted(
      ConventionPolicy,
      ctx,
      &ConventionAction::ViewAttendees,
      |ctx| Ok(ctx.data::<QueryData>()?.convention()),
    )
    .await
  }

  #[graphql(name = "can_create_cms_files")]
  async fn can_create_cms_files(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    let convention = ctx.data::<QueryData>()?.convention();
    let authorization_info = ctx.data::<AuthorizationInfo>()?;

    Ok(if let Some(convention) = convention {
      CmsContentPolicy::action_permitted(authorization_info, &ReadManageAction::Manage, convention)
        .await?
    } else {
      CmsContentPolicy::action_permitted(
        authorization_info,
        &ReadManageAction::Manage,
        &root_sites::Model {
          ..Default::default()
        },
      )
      .await?
    })
  }

  #[graphql(name = "can_create_user_con_profiles")]
  async fn can_create_user_con_profiles(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    let convention = ctx.data::<QueryData>()?.convention();

    let Some(convention) = convention else { return Ok(false); };

    let user_con_profile = user_con_profiles::Model {
      convention_id: convention.id,
      ..Default::default()
    };

    model_action_permitted(
      UserConProfilePolicy,
      ctx,
      &UserConProfileAction::Create,
      |_ctx| Ok(Some(user_con_profile)),
    )
    .await
  }

  #[graphql(name = "can_update_convention")]
  async fn can_update_convention(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    model_action_permitted(ConventionPolicy, ctx, &ConventionAction::Update, |ctx| {
      Ok(ctx.data::<QueryData>()?.convention())
    })
    .await
  }
  #[graphql(name = "can_update_departments")]
  async fn can_update_departments(&self) -> bool {
    false
  }
  #[graphql(name = "can_manage_email_routes")]
  async fn can_manage_email_routes(&self) -> bool {
    false
  }

  #[graphql(name = "can_update_event")]
  async fn can_update_event(&self, ctx: &Context<'_>, event_id: ID) -> Result<bool, Error> {
    let db = ctx.data::<QueryData>()?.db();
    let event = events::Entity::find_by_id(LaxId::parse(event_id)?)
      .one(db)
      .await?;

    let resource = if let Some(event) = event {
      Some((
        ctx
          .data::<QueryData>()?
          .loaders()
          .event_convention()
          .load_one(event.id)
          .await?
          .expect_one()?
          .clone(),
        event,
      ))
    } else {
      None
    };

    model_action_permitted(EventPolicy, ctx, &EventAction::Update, |_ctx| Ok(resource)).await
  }

  #[graphql(name = "can_delete_event")]
  async fn can_delete_event(&self, _event_id: ID) -> Result<bool, Error> {
    Ok(false)
  }

  #[graphql(name = "can_override_maximum_event_provided_tickets")]
  async fn can_override_maximum_event_provided_tickets(
    &self,
    ctx: &Context<'_>,
  ) -> Result<bool, Error> {
    let convention = ctx.data::<QueryData>()?.convention();
    let Some(convention) = convention else {
      return Ok(false);
    };

    let event = events::Model {
      id: 0,
      convention_id: convention.id,
      event_category_id: 0,
      can_play_concurrently: false,
      private_signup_list: false,
      length_seconds: 3600,
      status: "active".to_string(),
      title: "Null event".to_string(),
      ..Default::default()
    };

    let resource = (convention.clone(), event);

    model_action_permitted(
      EventPolicy,
      ctx,
      &EventAction::OverrideMaximumEventProvidedTickets,
      |_ctx| Ok(Some(resource)),
    )
    .await
  }

  #[graphql(name = "can_update_event_categories")]
  async fn can_update_event_categories(&self) -> bool {
    false
  }

  #[graphql(name = "can_read_event_proposals")]
  async fn can_read_event_proposals(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    model_action_permitted(
      ConventionPolicy,
      ctx,
      &ConventionAction::ViewEventProposals,
      |ctx| Ok(ctx.data::<QueryData>()?.convention()),
    )
    .await
  }

  #[graphql(name = "can_read_event_signups")]
  async fn can_read_event_signups(&self, ctx: &Context<'_>, event_id: ID) -> Result<bool, Error> {
    let db = ctx.data::<QueryData>()?.db();
    let event = events::Entity::find_by_id(LaxId::parse(event_id)?)
      .one(db)
      .await?;

    let resource = if let Some(event) = event {
      Some((
        ctx
          .data::<QueryData>()?
          .loaders()
          .event_convention()
          .load_one(event.id)
          .await?
          .expect_one()?
          .clone(),
        event,
      ))
    } else {
      None
    };

    model_action_permitted(EventPolicy, ctx, &EventAction::ReadSignups, |_ctx| {
      Ok(resource)
    })
    .await
  }

  #[graphql(name = "can_manage_runs")]
  async fn can_manage_runs(&self) -> bool {
    false
  }
  #[graphql(name = "can_manage_forms")]
  async fn can_manage_forms(&self) -> bool {
    false
  }
  #[graphql(name = "can_read_any_mailing_list")]
  async fn can_read_any_mailing_list(&self) -> bool {
    false
  }
  #[graphql(name = "can_update_notification_templates")]
  async fn can_update_notification_templates(&self) -> bool {
    false
  }
  #[graphql(name = "can_manage_oauth_applications")]
  async fn can_manage_oauth_applications(&self) -> bool {
    false
  }

  #[graphql(name = "can_read_reports")]
  async fn can_read_reports(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    model_action_permitted(
      ConventionPolicy,
      ctx,
      &ConventionAction::ViewReports,
      |ctx| Ok(ctx.data::<QueryData>()?.convention()),
    )
    .await
  }

  #[graphql(name = "can_manage_rooms")]
  async fn can_manage_rooms(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    model_action_permitted(RoomPolicy, ctx, &ReadManageAction::Manage, |ctx| {
      Ok(Some(rooms::Model {
        convention_id: ctx.data::<QueryData>()?.convention().map(|con| con.id),
        ..Default::default()
      }))
    })
    .await
  }

  #[graphql(name = "can_manage_signups")]
  async fn can_manage_signups(&self) -> bool {
    false
  }
  #[graphql(name = "can_manage_any_cms_content")]
  async fn can_manage_any_cms_content(&self) -> bool {
    false
  }
  #[graphql(name = "can_manage_staff_positions")]
  async fn can_manage_staff_positions(&self) -> bool {
    false
  }
  #[graphql(name = "can_read_orders")]
  async fn can_read_orders(&self) -> bool {
    false
  }
  #[graphql(name = "can_manage_ticket_types")]
  async fn can_manage_ticket_types(&self) -> bool {
    false
  }
  #[graphql(name = "can_read_user_activity_alerts")]
  async fn can_read_user_activity_alerts(&self) -> bool {
    false
  }
  #[graphql(name = "can_read_organizations")]
  async fn can_read_organizations(&self) -> bool {
    false
  }
  #[graphql(name = "can_read_users")]
  async fn can_read_users(&self) -> bool {
    false
  }

  #[graphql(name = "can_force_confirm_signup")]
  async fn can_force_confirm_signup(
    &self,
    ctx: &Context<'_>,
    signup_id: ID,
  ) -> Result<bool, Error> {
    let policy_model = self.get_signup_policy_model(ctx, signup_id).await?;

    model_action_permitted(SignupPolicy, ctx, &SignupAction::ForceConfirm, |_ctx| {
      Ok(Some(&policy_model))
    })
    .await
  }

  #[graphql(name = "can_update_bucket_signup")]
  async fn can_update_bucket_signup(
    &self,
    ctx: &Context<'_>,
    signup_id: ID,
  ) -> Result<bool, Error> {
    let policy_model = self.get_signup_policy_model(ctx, signup_id).await?;

    model_action_permitted(SignupPolicy, ctx, &SignupAction::UpdateBucket, |_ctx| {
      Ok(Some(&policy_model))
    })
    .await
  }

  #[graphql(name = "can_update_counted_signup")]
  async fn can_update_counted_signup(
    &self,
    ctx: &Context<'_>,
    signup_id: ID,
  ) -> Result<bool, Error> {
    let policy_model = self.get_signup_policy_model(ctx, signup_id).await?;

    model_action_permitted(SignupPolicy, ctx, &SignupAction::UpdateCounted, |_ctx| {
      Ok(Some(&policy_model))
    })
    .await
  }
}

impl AbilityType {
  async fn get_signup_policy_model(
    &self,
    ctx: &Context<'_>,
    signup_id: ID,
  ) -> Result<(events::Model, runs::Model, signups::Model), Error> {
    let query_data = ctx.data::<QueryData>()?;
    let signup = signups::Entity::find_by_id(signup_id.parse()?)
      .one(query_data.db())
      .await?
      .ok_or_else(|| Error::new("Signup not found"))?;

    let run_result = query_data
      .loaders()
      .signup_run()
      .load_one(signup.id)
      .await?;
    let run = run_result.expect_one()?;

    let event_result = query_data.loaders().run_event().load_one(run.id).await?;
    let event = event_result.expect_one()?;

    Ok((event.clone(), run.clone(), signup))
  }
}
