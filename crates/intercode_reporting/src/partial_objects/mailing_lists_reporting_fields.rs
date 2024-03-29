use std::{collections::HashMap, fmt};

use async_graphql::*;
use intercode_entities::{
  conventions, event_proposals, events,
  links::ConventionToSignups,
  model_ext::{
    event_proposals::EventProposalStatus,
    order_by_title::{NormalizedTitle, OrderByTitle},
    user_con_profiles::BioEligibility,
  },
  runs, signups, team_members, tickets, user_con_profiles, users, UserNames,
};
use intercode_graphql_core::{
  load_many_by_ids, load_many_by_model_ids, loader_result_map_to_required_map, model_backed_type,
  query_data::QueryData, scalars::DateScalar,
};
use intercode_policies::{
  policies::{ConventionAction, ConventionPolicy},
  ModelBackedTypeGuardablePolicy,
};
use itertools::Itertools;
use sea_orm::{
  prelude::SeaRc,
  sea_query::{self, Cond, Expr, Func, SimpleExpr},
  ColumnTrait, EntityTrait, Iden, ModelTrait, Order, QueryFilter, QueryOrder, QuerySelect,
};
use seawater::loaders::ExpectModel;

use crate::objects::{ContactEmail, ContactEmailType};

struct TrimFunction;

impl Iden for TrimFunction {
  fn unquoted(&self, s: &mut dyn fmt::Write) {
    s.write_str("TRIM").unwrap();
  }
}

pub struct MailingListsWaitlistsResult {
  pub emails: Vec<ContactEmailType>,
  pub run: runs::Model,
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

model_backed_type!(MailingListsReportingFields, conventions::Model);

pub async fn waitlists<T: From<MailingListsWaitlistsResult>>(
  model: &conventions::Model,
  ctx: &Context<'_>,
) -> Result<Vec<T>> {
  let db = ctx.data::<QueryData>()?.db();
  let signups = model
    .find_linked(ConventionToSignups)
    .filter(signups::Column::State.eq("waitlisted"))
    .all(db)
    .await?;

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
    .map(|(run_id, signups)| MailingListsWaitlistsResult {
      emails: signups
        .iter()
        .map(|signup| {
          let user_con_profile = user_con_profiles_by_signup_id.get(&signup.id).unwrap();
          let user = users_by_user_con_profile_id
            .get(&user_con_profile.id)
            .unwrap();
          ContactEmailType(ContactEmail::new(
            user.email.clone(),
            user_con_profile.name_inverted(),
            Some(user_con_profile.name_without_nickname()),
            std::iter::empty(),
          ))
        })
        .collect(),
      run: (*runs_by_id.get(run_id).unwrap()).clone(),
    })
    .collect::<Vec<_>>();

  results.sort_by_key(|result| (result.run.starts_at, result.run.id));
  Ok(results.into_iter().map(T::from).collect())
}

#[Object]
impl MailingListsReportingFields {
  #[graphql(
    name = "event_proposers",
    guard = "ConventionPolicy::model_guard(ConventionAction::ReadTeamMembersMailingList, self)"
  )]
  async fn event_proposers(&self, ctx: &Context<'_>) -> Result<MailingListsResult> {
    let query_data = ctx.data::<QueryData>()?;
    let results = self
      .model
      .find_related(event_proposals::Entity)
      .filter(
        event_proposals::Column::Status.is_not_in(
          [
            EventProposalStatus::Draft,
            EventProposalStatus::Rejected,
            EventProposalStatus::Withdrawn,
          ]
          .into_iter(),
        ),
      )
      .order_by_title(Order::Asc)
      .find_also_related(user_con_profiles::Entity)
      .all(query_data.db())
      .await?;

    let user_result = load_many_by_ids!(
      user_con_profile_user,
      ctx,
      results
        .iter()
        .filter_map(|(_event_proposal, user_con_profile)| user_con_profile
          .as_ref()
          .map(|ucp| ucp.id))
    )?;
    let users_by_user_con_profile_id = loader_result_map_to_required_map!(user_result)?;

    Ok(MailingListsResult::EventProposers(
      results
        .into_iter()
        .filter_map(|(event_proposal, user_con_profile)| {
          user_con_profile.map(|user_con_profile| {
            let user = users_by_user_con_profile_id
              .get(&user_con_profile.id)
              .unwrap();

            ContactEmail::new(
              user.email.clone(),
              user_con_profile.name_without_nickname(),
              None,
              [(
                "title".to_string(),
                serde_json::Value::String(event_proposal.title.unwrap_or_default()),
              )]
              .into_iter(),
            )
          })
        })
        .map(ContactEmailType)
        .collect(),
    ))
  }

  #[graphql(
    name = "team_members",
    guard = "ConventionPolicy::model_guard(ConventionAction::ReadTeamMembersMailingList, self)"
  )]
  async fn team_members(&self, ctx: &Context<'_>) -> Result<MailingListsResult> {
    let query_data = ctx.data::<QueryData>()?;
    let results = team_members::Entity::find()
      .inner_join(events::Entity)
      .filter(events::Column::ConventionId.eq(self.model.id))
      .filter(events::Column::Status.eq("active"))
      .order_by(events::Entity::normalized_title(), Order::Asc)
      .find_also_related(user_con_profiles::Entity)
      .all(query_data.db())
      .await?;

    let user_result = load_many_by_ids!(
      user_con_profile_user,
      ctx,
      results
        .iter()
        .filter_map(|(_team_member, user_con_profile)| user_con_profile.as_ref().map(|ucp| ucp.id))
    )?;
    let users_by_user_con_profile_id = loader_result_map_to_required_map!(user_result)?;

    let events_by_id = events::Entity::find()
      .filter(
        events::Column::Id.is_in(
          results
            .iter()
            .map(|(team_member, _ucp)| team_member.event_id),
        ),
      )
      .all(query_data.db())
      .await?
      .into_iter()
      .map(|event| (event.id, event))
      .collect::<HashMap<_, _>>();

    Ok(MailingListsResult::TeamMembers(
      results
        .into_iter()
        .filter_map(|(team_member, user_con_profile)| {
          user_con_profile.map(|user_con_profile| {
            let user = users_by_user_con_profile_id
              .get(&user_con_profile.id)
              .unwrap();

            let event = events_by_id.get(&team_member.event_id.unwrap()).unwrap();

            ContactEmail::new(
              user.email.clone(),
              user_con_profile.name_without_nickname(),
              None,
              [(
                "event".to_string(),
                serde_json::Value::String(event.title.clone()),
              )]
              .into_iter(),
            )
          })
        })
        .map(ContactEmailType)
        .collect(),
    ))
  }

  #[graphql(
    name = "ticketed_attendees",
    guard = "ConventionPolicy::model_guard(ConventionAction::ReadUserConProfilesMailingList, self)"
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
              Some(user_con_profile.name_without_nickname()),
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
    guard = "ConventionPolicy::model_guard(ConventionAction::ReadTeamMembersMailingList, self)"
  )]
  async fn users_with_pending_bio(&self, ctx: &Context<'_>) -> Result<MailingListsResult> {
    let query_data = ctx.data::<QueryData>()?;
    let results = self
      .model
      .find_related(user_con_profiles::Entity)
      .bio_eligible()
      .filter(
        Cond::any()
          .add(user_con_profiles::Column::Bio.is_null())
          .add(
            SimpleExpr::FunctionCall(
              Func::cust(TrimFunction).arg(Expr::col(user_con_profiles::Column::Bio)),
            )
            .eq(""),
          ),
      )
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
              Some(user_con_profile.name_without_nickname()),
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
    name = "whos_free",
    guard = "ConventionPolicy::model_guard(ConventionAction::ReadUserConProfilesMailingList, self)"
  )]
  async fn whos_free(
    &self,
    ctx: &Context<'_>,
    start: DateScalar,
    finish: DateScalar,
  ) -> Result<MailingListsResult> {
    let db = ctx.data::<QueryData>()?.db();
    let runs_in_timespan = runs::Entity::find().filter(Expr::cust_with_exprs(
      "tsrange($1, $2, '[)') && $3",
      [
        SimpleExpr::Value(start.0.naive_utc().into()),
        SimpleExpr::Value(finish.0.naive_utc().into()),
        SimpleExpr::Column(sea_query::ColumnRef::TableColumn(
          SeaRc::new(runs::Entity),
          SeaRc::new(runs::Column::TimespanTsrange),
        )),
      ]
      .into_iter(),
    ));

    let signups_during_timespan = self
      .model
      .find_linked(ConventionToSignups)
      .filter(signups::Column::State.ne("withdrawn"))
      .filter(signups::Column::RunId.in_subquery(
        QuerySelect::query(&mut runs_in_timespan.select_only().column(runs::Column::Id)).take(),
      ));

    let ticketed_user_con_profiles = self
      .model
      .find_related(user_con_profiles::Entity)
      .inner_join(tickets::Entity)
      .filter(user_con_profiles::Column::ReceiveWhosFreeEmails.eq(true));

    let free_user_con_profiles = ticketed_user_con_profiles.filter(
      user_con_profiles::Column::Id.not_in_subquery(
        QuerySelect::query(
          &mut signups_during_timespan
            .select_only()
            .column(signups::Column::UserConProfileId),
        )
        .take(),
      ),
    );

    Ok(MailingListsResult::WhosFree(
      free_user_con_profiles
        .find_also_related(users::Entity)
        .all(db)
        .await?
        .into_iter()
        .filter_map(|(user_con_profile, user)| {
          user.map(|user| {
            ContactEmailType(ContactEmail::new(
              user.email,
              user_con_profile.name_inverted(),
              Some(user_con_profile.name_without_nickname()),
              std::iter::empty(),
            ))
          })
        })
        .collect(),
    ))
  }
}
