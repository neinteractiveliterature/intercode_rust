use std::borrow::{Borrow, Cow};

use async_graphql::*;
use intercode_entities::{
  cms_content_model::CmsContentModel, conventions, events, orders, pages, products, rooms,
  root_sites, runs, signups, staff_positions, ticket_types, tickets, user_con_profiles,
};
use intercode_policies::{
  policies::{
    CmsContentPolicy, ConventionAction, ConventionPolicy, EventAction, EventPolicy,
    EventProposalAction, EventProposalPolicy, OrderAction, OrderPolicy, ProductPolicy, RoomPolicy,
    RunAction, RunPolicy, SignupAction, SignupPolicy, StaffPositionPolicy, TicketAction,
    TicketPolicy, TicketTypePolicy, UserConProfileAction, UserConProfilePolicy,
  },
  AuthorizationInfo, EntityPolicy, Policy, ReadManageAction,
};
use sea_orm::{EntityTrait, PaginatorTrait};
use seawater::loaders::ExpectModel;

use crate::{lax_id::LaxId, load_one_by_id, QueryData};

pub struct AbilityType<'a> {
  authorization_info: Cow<'a, AuthorizationInfo>,
}

async fn model_action_permitted<
  'a,
  P: Policy<AuthorizationInfo, M>,
  M: Send + Sync + 'a,
  R: Borrow<M>,
>(
  authorization_info: &AuthorizationInfo,
  _policy: P,
  ctx: &'a Context<'_>,
  action: &P::Action,
  get_model: impl FnOnce(&'a Context<'_>) -> Result<Option<R>, Error>,
) -> Result<bool, Error> {
  let model_ref = get_model(ctx)?;

  if let Some(model_ref) = model_ref {
    Ok(P::action_permitted(authorization_info, action, model_ref.borrow()).await?)
  } else {
    Ok(false)
  }
}

impl<'a> AbilityType<'a> {
  pub fn new(authorization_info: Cow<'a, AuthorizationInfo>) -> Self {
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
    let signup = signups::Entity::find_by_id(LaxId::parse(signup_id)?)
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

    let convention_result = query_data
      .loaders()
      .event_convention()
      .load_one(event.id)
      .await?;
    let convention = convention_result.expect_one()?;

    Ok((convention.clone(), event.clone(), run.clone(), signup))
  }

  async fn get_ticket_policy_model(
    &self,
    ctx: &Context<'_>,
    ticket_id: ID,
  ) -> Result<(conventions::Model, user_con_profiles::Model, tickets::Model), Error> {
    let query_data = ctx.data::<QueryData>()?;
    let ticket = tickets::Entity::find_by_id(LaxId::parse(ticket_id)?)
      .one(query_data.db())
      .await?
      .ok_or_else(|| Error::new("Ticket not found"))?;

    let user_con_profile_result = query_data
      .loaders()
      .ticket_user_con_profile()
      .load_one(ticket.id)
      .await?;
    let user_con_profile = user_con_profile_result.expect_one()?;

    let convention_result = query_data
      .loaders()
      .user_con_profile_convention()
      .load_one(user_con_profile.id)
      .await?;
    let convention = convention_result.expect_one()?;

    Ok((convention.clone(), user_con_profile.clone(), ticket))
  }

  async fn can_perform_event_proposal_action(
    &self,
    ctx: &Context<'_>,
    event_proposal_id: ID,
    action: &EventProposalAction,
  ) -> Result<bool> {
    let query_data = ctx.data::<QueryData>()?;
    let loader_result = query_data
      .loaders()
      .event_proposals_by_id()
      .load_one(LaxId::parse(event_proposal_id)?)
      .await?;

    let event_proposal = loader_result.expect_one()?;
    let convention_result = query_data
      .loaders()
      .event_proposal_convention()
      .load_one(event_proposal.id)
      .await?;
    let convention = convention_result.expect_one()?;

    model_action_permitted(
      self.authorization_info.as_ref(),
      EventProposalPolicy,
      ctx,
      action,
      |_ctx| Ok(Some((convention.clone(), event_proposal.clone()))),
    )
    .await
  }

  async fn can_perform_user_con_profile_action(
    &self,
    ctx: &Context<'_>,
    user_con_profile_id: ID,
    action: &UserConProfileAction,
  ) -> Result<bool> {
    let loader_result = ctx
      .data::<QueryData>()?
      .loaders()
      .user_con_profiles_by_id()
      .load_one(LaxId::parse(user_con_profile_id)?)
      .await?;

    let user_con_profile = loader_result.expect_one()?;

    model_action_permitted(
      self.authorization_info.as_ref(),
      UserConProfilePolicy,
      ctx,
      action,
      |_ctx| Ok(Some(user_con_profile)),
    )
    .await
  }

  async fn can_perform_cms_content_action(
    &self,
    ctx: &Context<'_>,
    action: ReadManageAction,
  ) -> Result<bool, Error> {
    let convention = ctx.data::<QueryData>()?.convention();

    Ok(if let Some(convention) = convention {
      CmsContentPolicy::action_permitted(self.authorization_info.as_ref(), &action, convention)
        .await?
    } else {
      CmsContentPolicy::action_permitted(
        self.authorization_info.as_ref(),
        &action,
        &root_sites::Model {
          ..Default::default()
        },
      )
      .await?
    })
  }
}

#[Object(name = "Ability")]
impl<'a> AbilityType<'a> {
  #[graphql(name = "can_manage_conventions")]
  async fn can_manage_conventions(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    model_action_permitted(
      self.authorization_info.as_ref(),
      ConventionPolicy,
      ctx,
      &ConventionAction::Update,
      |_ctx| Ok(Some(conventions::Model::default())),
    )
    .await
  }

  #[graphql(name = "can_read_schedule")]
  async fn can_read_schedule(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    model_action_permitted(
      self.authorization_info.as_ref(),
      ConventionPolicy,
      ctx,
      &ConventionAction::Schedule,
      |ctx| Ok(ctx.data::<QueryData>()?.convention()),
    )
    .await
  }

  #[graphql(name = "can_read_schedule_with_counts")]
  async fn can_read_schedule_with_counts(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    model_action_permitted(
      self.authorization_info.as_ref(),
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
      self.authorization_info.as_ref(),
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
      self.authorization_info.as_ref(),
      ConventionPolicy,
      ctx,
      &ConventionAction::ViewAttendees,
      |ctx| Ok(ctx.data::<QueryData>()?.convention()),
    )
    .await
  }

  #[graphql(name = "can_create_cms_files")]
  async fn can_create_cms_files(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    self
      .can_perform_cms_content_action(ctx, ReadManageAction::Manage)
      .await
  }

  #[graphql(name = "can_create_pages")]
  async fn can_create_pages(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    self
      .can_perform_cms_content_action(ctx, ReadManageAction::Manage)
      .await
  }

  #[graphql(name = "can_create_cms_partials")]
  async fn can_create_cms_partials(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    self
      .can_perform_cms_content_action(ctx, ReadManageAction::Manage)
      .await
  }

  #[graphql(name = "can_create_cms_layouts")]
  async fn can_create_cms_layouts(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    self
      .can_perform_cms_content_action(ctx, ReadManageAction::Manage)
      .await
  }

  #[graphql(name = "can_create_cms_navigation_items")]
  async fn can_create_cms_navigation_items(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    self
      .can_perform_cms_content_action(ctx, ReadManageAction::Manage)
      .await
  }

  #[graphql(name = "can_create_cms_variables")]
  async fn can_create_cms_variables(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    self
      .can_perform_cms_content_action(ctx, ReadManageAction::Manage)
      .await
  }

  #[graphql(name = "can_create_cms_graphql_queries")]
  async fn can_create_cms_graphql_queries(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    self
      .can_perform_cms_content_action(ctx, ReadManageAction::Manage)
      .await
  }

  #[graphql(name = "can_create_cms_content_groups")]
  async fn can_create_cms_content_groups(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    self
      .can_perform_cms_content_action(ctx, ReadManageAction::Manage)
      .await
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
      self.authorization_info.as_ref(),
      UserConProfilePolicy,
      ctx,
      &UserConProfileAction::Create,
      |_ctx| Ok(Some(user_con_profile)),
    )
    .await
  }

  #[graphql(name = "can_become_user_con_profile")]
  async fn can_become_user_con_profile(
    &self,
    ctx: &Context<'_>,
    user_con_profile_id: ID,
  ) -> Result<bool, Error> {
    self
      .can_perform_user_con_profile_action(ctx, user_con_profile_id, &UserConProfileAction::Become)
      .await
  }

  #[graphql(name = "can_delete_user_con_profile")]
  async fn can_delete_user_con_profile(
    &self,
    ctx: &Context<'_>,
    user_con_profile_id: ID,
  ) -> Result<bool, Error> {
    self
      .can_perform_user_con_profile_action(ctx, user_con_profile_id, &UserConProfileAction::Delete)
      .await
  }

  #[graphql(name = "can_update_user_con_profile")]
  async fn can_update_user_con_profile(
    &self,
    ctx: &Context<'_>,
    user_con_profile_id: ID,
  ) -> Result<bool, Error> {
    self
      .can_perform_user_con_profile_action(ctx, user_con_profile_id, &UserConProfileAction::Update)
      .await
  }

  #[graphql(name = "can_withdraw_all_user_con_profile_signups")]
  async fn can_withdraw_all_user_con_profile_signups(
    &self,
    ctx: &Context<'_>,
    user_con_profile_id: ID,
  ) -> Result<bool, Error> {
    self
      .can_perform_user_con_profile_action(
        ctx,
        user_con_profile_id,
        &UserConProfileAction::WithdrawAllSignups,
      )
      .await
  }

  #[graphql(name = "can_update_convention")]
  async fn can_update_convention(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    model_action_permitted(
      self.authorization_info.as_ref(),
      ConventionPolicy,
      ctx,
      &ConventionAction::Update,
      |ctx| Ok(ctx.data::<QueryData>()?.convention()),
    )
    .await
  }
  #[graphql(name = "can_update_departments")]
  async fn can_update_departments(&self) -> bool {
    // TODO
    false
  }
  #[graphql(name = "can_manage_email_routes")]
  async fn can_manage_email_routes(&self) -> bool {
    // TODO
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

    model_action_permitted(
      self.authorization_info.as_ref(),
      EventPolicy,
      ctx,
      &EventAction::Update,
      |_ctx| Ok(resource),
    )
    .await
  }

  #[graphql(
    name = "can_delete_event",
    deprecation = "Deleting events is never allowed; this always returns false"
  )]
  async fn can_delete_event(&self, _event_id: ID) -> Result<bool, Error> {
    Ok(false)
  }

  #[graphql(name = "can_read_admin_notes_on_event_proposal")]
  async fn can_read_admin_notes_on_event_proposal(
    &self,
    ctx: &Context<'_>,
    event_proposal_id: ID,
  ) -> Result<bool, Error> {
    self
      .can_perform_event_proposal_action(
        ctx,
        event_proposal_id,
        &EventProposalAction::ReadAdminNotes,
      )
      .await
  }

  #[graphql(name = "can_update_event_proposal")]
  async fn can_update_event_proposal(
    &self,
    ctx: &Context<'_>,
    event_proposal_id: ID,
  ) -> Result<bool, Error> {
    self
      .can_perform_event_proposal_action(ctx, event_proposal_id, &EventProposalAction::Update)
      .await
  }

  #[graphql(name = "can_delete_event_proposal")]
  async fn can_delete_event_proposal(
    &self,
    ctx: &Context<'_>,
    event_proposal_id: ID,
  ) -> Result<bool, Error> {
    self
      .can_perform_event_proposal_action(ctx, event_proposal_id, &EventProposalAction::Delete)
      .await
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
      self.authorization_info.as_ref(),
      EventPolicy,
      ctx,
      &EventAction::OverrideMaximumEventProvidedTickets,
      |_ctx| Ok(Some(resource)),
    )
    .await
  }

  #[graphql(name = "can_update_event_categories")]
  async fn can_update_event_categories(&self) -> bool {
    // TODO
    false
  }

  #[graphql(name = "can_read_event_proposals")]
  async fn can_read_event_proposals(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    model_action_permitted(
      self.authorization_info.as_ref(),
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

    model_action_permitted(
      self.authorization_info.as_ref(),
      EventPolicy,
      ctx,
      &EventAction::ReadSignups,
      |_ctx| Ok(resource),
    )
    .await
  }

  #[graphql(name = "can_manage_runs")]
  async fn can_manage_runs(&self, ctx: &Context<'_>) -> Result<bool> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let convention = ctx.data::<QueryData>()?.convention();
    let Some(convention) = convention else {
      return Ok(false);
    };

    Ok(
      RunPolicy::action_permitted(
        authorization_info,
        &RunAction::Manage,
        &(
          convention.clone(),
          events::Model {
            convention_id: convention.id,
            ..Default::default()
          },
          runs::Model::default(),
        ),
      )
      .await?,
    )
  }

  #[graphql(name = "can_manage_forms")]
  async fn can_manage_forms(&self) -> bool {
    // TODO
    false
  }
  #[graphql(name = "can_read_any_mailing_list")]
  async fn can_read_any_mailing_list(&self) -> bool {
    // TODO
    false
  }
  #[graphql(name = "can_update_notification_templates")]
  async fn can_update_notification_templates(&self) -> bool {
    // TODO
    false
  }
  #[graphql(name = "can_manage_oauth_applications")]
  async fn can_manage_oauth_applications(&self) -> bool {
    // TODO
    false
  }

  #[graphql(name = "can_read_reports")]
  async fn can_read_reports(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    model_action_permitted(
      self.authorization_info.as_ref(),
      ConventionPolicy,
      ctx,
      &ConventionAction::ViewReports,
      |ctx| Ok(ctx.data::<QueryData>()?.convention()),
    )
    .await
  }

  #[graphql(name = "can_update_products")]
  async fn can_update_products(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    model_action_permitted(
      self.authorization_info.as_ref(),
      ProductPolicy,
      ctx,
      &ReadManageAction::Manage,
      |ctx| {
        Ok(Some(products::Model {
          convention_id: ctx.data::<QueryData>()?.convention().map(|con| con.id),
          ..Default::default()
        }))
      },
    )
    .await
  }

  #[graphql(name = "can_manage_rooms")]
  async fn can_manage_rooms(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    model_action_permitted(
      self.authorization_info.as_ref(),
      RoomPolicy,
      ctx,
      &ReadManageAction::Manage,
      |ctx| {
        Ok(Some(rooms::Model {
          convention_id: ctx.data::<QueryData>()?.convention().map(|con| con.id),
          ..Default::default()
        }))
      },
    )
    .await
  }

  #[graphql(name = "can_manage_signups")]
  async fn can_manage_signups(&self, ctx: &Context<'_>) -> Result<bool> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
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

  #[graphql(name = "can_manage_any_cms_content")]
  async fn can_manage_any_cms_content(&self, ctx: &Context<'_>) -> Result<bool> {
    let query_data = ctx.data::<QueryData>()?;

    Ok(
      pages::Model::filter_by_parent(
        CmsContentPolicy::<pages::Model>::accessible_to(
          &self.authorization_info,
          &ReadManageAction::Manage,
        ),
        query_data.cms_parent(),
      )
      .count(query_data.db())
      .await?
        > 0,
    )
  }

  #[graphql(name = "can_manage_staff_positions")]
  async fn can_manage_staff_positions(&self, ctx: &Context<'_>) -> Result<bool> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let convention = ctx.data::<QueryData>()?.convention();
    Ok(
      StaffPositionPolicy::action_permitted(
        authorization_info,
        &ReadManageAction::Manage,
        &staff_positions::Model {
          convention_id: convention.map(|c| c.id),
          ..Default::default()
        },
      )
      .await?,
    )
  }

  #[graphql(name = "can_read_orders")]
  async fn can_read_orders(&self, ctx: &Context<'_>) -> Result<bool> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let convention = ctx.data::<QueryData>()?.convention();
    let Some(convention)= convention else {
      return Ok(false);
    };

    Ok(
      OrderPolicy::action_permitted(
        authorization_info,
        &OrderAction::Read,
        &(
          convention.clone(),
          user_con_profiles::Model {
            convention_id: convention.id,
            ..Default::default()
          },
          orders::Model {
            ..Default::default()
          },
          vec![],
        ),
      )
      .await?,
    )
  }

  #[graphql(name = "can_create_orders")]
  async fn can_create_orders(&self, ctx: &Context<'_>) -> Result<bool> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let convention = ctx.data::<QueryData>()?.convention();
    let Some(convention)= convention else {
      return Ok(false);
    };

    Ok(
      OrderPolicy::action_permitted(
        authorization_info,
        &OrderAction::Manage,
        &(
          convention.clone(),
          user_con_profiles::Model {
            convention_id: convention.id,
            ..Default::default()
          },
          orders::Model {
            ..Default::default()
          },
          vec![],
        ),
      )
      .await?,
    )
  }

  #[graphql(name = "can_update_orders")]
  async fn can_update_orders(&self, ctx: &Context<'_>) -> Result<bool> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let convention = ctx.data::<QueryData>()?.convention();
    let Some(convention)= convention else {
      return Ok(false);
    };

    Ok(
      OrderPolicy::action_permitted(
        authorization_info,
        &OrderAction::Manage,
        &(
          convention.clone(),
          user_con_profiles::Model {
            convention_id: convention.id,
            ..Default::default()
          },
          orders::Model {
            ..Default::default()
          },
          vec![],
        ),
      )
      .await?,
    )
  }

  #[graphql(name = "can_manage_ticket_types")]
  async fn can_manage_ticket_types(&self, ctx: &Context<'_>) -> Result<bool> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let convention = ctx.data::<QueryData>()?.convention();
    let Some(convention)= convention else {
      return Ok(false);
    };
    let single_event_loader_result = load_one_by_id!(convention_single_event, ctx, convention.id)?;
    let single_event = single_event_loader_result.try_one();

    Ok(
      TicketTypePolicy::action_permitted(
        authorization_info,
        &ReadManageAction::Manage,
        &(
          convention.clone(),
          single_event.cloned(),
          ticket_types::Model {
            convention_id: Some(convention.id),
            ..Default::default()
          },
        ),
      )
      .await?,
    )
  }

  #[graphql(name = "can_read_user_activity_alerts")]
  async fn can_read_user_activity_alerts(&self) -> bool {
    // TODO
    false
  }
  #[graphql(name = "can_read_organizations")]
  async fn can_read_organizations(&self) -> bool {
    // TODO
    false
  }

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

  #[graphql(name = "can_create_tickets")]
  async fn can_create_tickets(&self, ctx: &Context<'_>) -> Result<bool> {
    let convention = ctx.data::<QueryData>()?.convention();

    if let Some(convention) = convention {
      let user_con_profile = user_con_profiles::Model {
        convention_id: convention.id,
        ..Default::default()
      };
      let ticket = tickets::Model {
        ..Default::default()
      };

      model_action_permitted(
        &self.authorization_info,
        TicketPolicy,
        ctx,
        &TicketAction::Manage,
        |_ctx| Ok(Some((convention.clone(), user_con_profile, ticket))),
      )
      .await
    } else {
      Ok(false)
    }
  }

  #[graphql(name = "can_delete_ticket")]
  async fn can_delete_ticket(&self, ctx: &Context<'_>, ticket_id: ID) -> Result<bool> {
    Ok(
      TicketPolicy::action_permitted(
        &self.authorization_info,
        &TicketAction::Manage,
        &(self.get_ticket_policy_model(ctx, ticket_id).await?),
      )
      .await?,
    )
  }

  #[graphql(name = "can_update_ticket")]
  async fn can_update_ticket(&self, ctx: &Context<'_>, ticket_id: ID) -> Result<bool> {
    Ok(
      TicketPolicy::action_permitted(
        &self.authorization_info,
        &TicketAction::Manage,
        &(self.get_ticket_policy_model(ctx, ticket_id).await?),
      )
      .await?,
    )
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
