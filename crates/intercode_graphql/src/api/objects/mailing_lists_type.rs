use std::collections::HashMap;

use async_graphql::*;
use intercode_entities::{
  conventions, model_ext::user_con_profiles::BioEligibility, runs, tickets, user_con_profiles,
  users, UserNames,
};
use intercode_policies::policies::{ConventionAction, ConventionPolicy};
use itertools::Itertools;
use sea_orm::ModelTrait;
use seawater::loaders::{ExpectModel, ExpectModels};

use crate::{
  api::objects::ModelBackedType, load_many_by_model_ids, load_one_by_id,
  loader_result_map_to_required_map, model_backed_type, QueryData,
};

pub struct MailingListsWaitlistResult {
  pub emails: Vec<ContactEmailType>,
  pub run: runs::Model,
}

#[Object]
impl MailingListsWaitlistResult {
  async fn emails(&self) -> &Vec<ContactEmailType> {
    &self.emails
  }

  #[graphql(name = "metadata_fields")]
  async fn metadata_fields(&self) -> &'static [&'static str] {
    &[]
  }

  async fn run(&self) -> RunType {
    RunType::new(self.run.clone())
  }
}

pub enum MailingListsResult {
  EventProposers(Vec<ContactEmailType>),
  TeamMembers(Vec<ContactEmailType>),
  TicketedAttendees(Vec<ContactEmailType>),
  UsersWithPendingBio(Vec<ContactEmailType>),
  WhosFree(Vec<ContactEmailType>),
}

#[Object]
impl MailingListsResult {
  async fn emails(&self) -> &Vec<ContactEmailType> {
    match self {
      Self::EventProposers(emails)
      | Self::TeamMembers(emails)
      | Self::TicketedAttendees(emails)
      | Self::UsersWithPendingBio(emails)
      | Self::WhosFree(emails) => emails,
    }
  }

  #[graphql(name = "metadata_fields")]
  async fn metadata_fields(&self) -> &'static [&'static str] {
    match self {
      MailingListsResult::EventProposers(_) => &["title"],
      MailingListsResult::TeamMembers(_) => &["event"],
      _ => &[],
    }
  }
}

use super::{
  contact_email_type::{ContactEmail, ContactEmailType},
  RunType,
};
model_backed_type!(MailingListsType, conventions::Model);

#[Object(name = "MailingLists")]
impl MailingListsType {
  #[graphql(
    name = "ticketed_attendees",
    guard = "self.simple_policy_guard::<ConventionPolicy>(ConventionAction::ReadUserConProfilesMailingList)"
  )]
  async fn ticketed_attendees(&self, ctx: &Context<'_>) -> Result<MailingListsResult> {
    let query_data = ctx.data::<QueryData>()?;
    let results = self
      .model
      .find_related(user_con_profiles::Entity)
      .inner_join(tickets::Entity)
      .find_also_related(users::Entity)
      .all(query_data.db())
      .await?;

    Ok(MailingListsResult::TicketedAttendees(
      results
        .into_iter()
        .filter_map(|(user_con_profile, user)| {
          user.map(|user| {
            ContactEmail::new(
              user.email,
              user_con_profile.name_inverted(),
              None,
              std::iter::empty(),
            )
          })
        })
        .sorted_by_key(|contact_email| contact_email.name.clone())
        .map(ContactEmailType)
        .collect(),
    ))
  }

  #[graphql(
    name = "users_with_pending_bio",
    guard = "self.simple_policy_guard::<ConventionPolicy>(ConventionAction::ReadUserConProfilesMailingList)"
  )]
  async fn users_with_pending_bio(&self, ctx: &Context<'_>) -> Result<MailingListsResult> {
    let query_data = ctx.data::<QueryData>()?;
    let results = self
      .model
      .find_related(user_con_profiles::Entity)
      .bio_eligible()
      .find_also_related(users::Entity)
      .all(query_data.db())
      .await?;

    Ok(MailingListsResult::UsersWithPendingBio(
      results
        .into_iter()
        .filter_map(|(user_con_profile, user)| {
          user.map(|user| {
            ContactEmail::new(
              user.email,
              user_con_profile.name_inverted(),
              None,
              std::iter::empty(),
            )
          })
        })
        .sorted_by_key(|contact_email| contact_email.name.clone())
        .map(ContactEmailType)
        .collect(),
    ))
  }

  async fn waitlists(&self, ctx: &Context<'_>) -> Result<Vec<MailingListsWaitlistResult>> {
    let signups_result = load_one_by_id!(convention_signups, ctx, self.model.id)?;
    let signups = signups_result.expect_models()?;

    let runs_by_signup_id_result = load_many_by_model_ids!(signup_run, ctx, signups.iter())?;
    let runs_by_signup_id = loader_result_map_to_required_map!(runs_by_signup_id_result)?;
    let runs_by_id = runs_by_signup_id
      .values()
      .map(|run| (run.id, run))
      .collect::<HashMap<_, _>>();

    let user_con_profiles_by_signup_id_result =
      load_many_by_model_ids!(signup_user_con_profile, ctx, signups.iter())?;
    let user_con_profiles_by_signup_id =
      loader_result_map_to_required_map!(user_con_profiles_by_signup_id_result)?;

    let users_by_user_con_profile_id_result = load_many_by_model_ids!(
      user_con_profile_user,
      ctx,
      user_con_profiles_by_signup_id.values()
    )?;
    let users_by_user_con_profile_id =
      loader_result_map_to_required_map!(users_by_user_con_profile_id_result)?;

    let signups_by_id = signups
      .iter()
      .map(|signup| (signup.id, signup))
      .collect::<HashMap<_, _>>();
    let signups_by_run_id = runs_by_signup_id
      .iter()
      .map(|(signup_id, run)| (run.id, signups_by_id.get(signup_id).unwrap()))
      .into_group_map();

    let mut results = signups_by_run_id
      .iter()
      .map(|(run_id, signups)| MailingListsWaitlistResult {
        emails: signups
          .iter()
          .map(|signup| {
            let user_con_profile = user_con_profiles_by_signup_id.get(&signup.id).unwrap();
            let user = users_by_user_con_profile_id
              .get(&user_con_profile.id)
              .unwrap();
            ContactEmailType(ContactEmail::new(
              user.email.clone(),
              user_con_profile.name(),
              None,
              std::iter::empty(),
            ))
          })
          .collect(),
        run: (*runs_by_id.get(run_id).unwrap()).clone(),
      })
      .collect::<Vec<_>>();

    results.sort_by_key(|result| (result.run.starts_at.unwrap_or_default(), result.run.id));
    Ok(results)
  }
}
