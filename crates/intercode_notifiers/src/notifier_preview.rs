use std::fmt::Display;

use async_graphql::Error;
use intercode_entities::{conventions, events, runs, signup_requests, user_con_profiles, users};
use intercode_liquid_drops::drops::{
  ConventionDrop, DropContext, EventDrop, RunDrop, SignupRequestDrop, UserConProfileDrop, UserDrop,
};
use liquid::object;
use seawater::{Context, DropResult, LiquidDrop, ModelBackedDrop};

use crate::{signup_requests::RequestAcceptedNotifier, Notifier};

pub struct UnknownNotifierKeyError(String);

impl Display for UnknownNotifierKeyError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!("Unknown notifier key: {}", self.0))
  }
}

fn mock_user_drop(ctx: DropContext) -> DropResult<UserDrop> {
  ctx.with_drop_store(|store| {
    let drop_ref = store.store(UserDrop::new(
      users::Model {
        first_name: "Firstname".to_string(),
        last_name: "Lastname".to_string(),
        email: "firstname@example.com".to_string(),
        ..Default::default()
      },
      ctx.clone(),
    ));

    DropResult::from(drop_ref)
  })
}

fn mock_user_con_profile_drop(ctx: DropContext) -> DropResult<UserConProfileDrop> {
  ctx.with_drop_store(|store| {
    let user = mock_user_drop(ctx.clone());
    let inner_user = user.get_inner_cloned().unwrap();
    let drop_ref = store.store(UserConProfileDrop::new(
      user_con_profiles::Model {
        first_name: inner_user.get_model().first_name.clone(),
        last_name: inner_user.get_model().last_name.clone(),
        ..Default::default()
      },
      ctx.clone(),
    ));
    let _ = store
      .get_drop_cache::<UserConProfileDrop>(drop_ref.id())
      .set_user(user);

    DropResult::from(drop_ref)
  })
}

fn mock_convention_drop(ctx: DropContext) -> DropResult<ConventionDrop> {
  ctx.with_drop_store(|store| {
    let convention = ctx.query_data().convention().cloned().unwrap_or_default();
    DropResult::from(store.store(ConventionDrop::new(convention, ctx.clone())))
  })
}

fn mock_event_drop(ctx: DropContext) -> DropResult<EventDrop> {
  ctx.with_drop_store(|store| {
    let drop_ref = store.store(EventDrop::new(
      events::Model {
        title: "Event Title".to_string(),
        ..Default::default()
      },
      ctx.clone(),
    ));

    DropResult::from(drop_ref)
  })
}

fn mock_run_drop(ctx: DropContext) -> DropResult<RunDrop> {
  ctx.with_drop_store(|store| {
    let drop_ref = store.store(RunDrop::new(
      runs::Model {
        ..Default::default()
      },
      ctx.clone(),
    ));

    let _ = store
      .get_drop_cache::<RunDrop>(drop_ref.id())
      .set_event(mock_event_drop(ctx.clone()));

    DropResult::from(drop_ref)
  })
}

fn mock_signup_request_drop(ctx: DropContext) -> DropResult<SignupRequestDrop> {
  ctx.with_drop_store(|store| {
    let drop_ref = store.store(SignupRequestDrop::new(
      signup_requests::Model::default(),
      ctx.clone(),
    ));
    let run = mock_run_drop(ctx.clone());
    let run_cache = store.get_drop_cache::<RunDrop>(run.get_inner_cloned().unwrap().id());
    let event = run_cache.event.get().unwrap();
    let cache = store.get_drop_cache::<SignupRequestDrop>(drop_ref.id());
    let _ = cache.set_user_con_profile(mock_user_con_profile_drop(ctx.clone()));
    let _ = cache.set_target_run(run);
    let _ = cache.set_event(event.clone());

    DropResult::from(drop_ref)
  })
}

pub fn build_notifier_preview(
  convention: &conventions::Model,
  event_key: &str,
  ctx: DropContext,
) -> Result<Box<dyn Notifier>, Error> {
  match event_key {
    "signup_requests/request_accepted" => {
      Ok(Box::new(RequestAcceptedNotifier::with_liquid_assigns(
        convention.clone(),
        signup_requests::Model::default(),
        object!({
          "signup_request": mock_signup_request_drop(ctx).get_inner_cloned()
        }),
      )))
    }
    _ => Err(UnknownNotifierKeyError(event_key.to_string()).into()),
  }
}
