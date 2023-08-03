use std::sync::Arc;

use async_graphql::*;
use intercode_entities::{
  conventions, event_categories, events, forms, tickets, RegistrationPolicy,
};
use intercode_graphql_core::{
  lax_id::LaxId, load_one_by_id, load_one_by_model_id, loader_result_to_many, model_backed_type,
  objects::ActiveStorageAttachmentType, query_data::QueryData, scalars::DateScalar,
  ModelBackedType,
};
use intercode_graphql_loaders::{
  attached_images_by_filename::attached_images_by_filename,
  filtered_event_runs_loader::EventRunsLoaderFilter, LoaderManager,
};
use intercode_liquid::render_markdown;
use intercode_policies::{
  policies::{EventAction, EventPolicy},
  ModelBackedTypeGuardablePolicy,
};
use seawater::loaders::ExpectModel;

use crate::objects::RegistrationPolicyType;

use super::{team_member_events_fields::TeamMemberEventsFields, RunEventsFields};

model_backed_type!(EventEventsFields, events::Model);

impl EventEventsFields {
  pub async fn convention(&self, ctx: &Context<'_>) -> Result<conventions::Model, Error> {
    let loader_result = load_one_by_id!(conventions_by_id, ctx, self.model.convention_id)?;
    Ok(loader_result.expect_one()?.clone())
  }

  pub async fn event_category(&self, ctx: &Context<'_>) -> Result<event_categories::Model, Error> {
    let loader_result = load_one_by_model_id!(event_event_category, ctx, self)?;
    Ok(loader_result.expect_one()?.clone())
  }

  pub async fn form(&self, ctx: &Context<'_>) -> Result<forms::Model, Error> {
    let event_category = self.event_category(ctx).await?;
    let loader_result = load_one_by_id!(event_category_event_form, ctx, event_category.id)?;
    Ok(loader_result.expect_one()?.clone())
  }

  pub async fn provided_tickets(&self, ctx: &Context<'_>) -> Result<Vec<tickets::Model>> {
    let loader_result = load_one_by_model_id!(event_provided_tickets, ctx, self)?;
    Ok(loader_result.expect_one().into_iter().cloned().collect())
  }

  pub async fn run(&self, ctx: &Context<'_>, id: ID) -> Result<RunEventsFields, Error> {
    let loaders = ctx.data::<Arc<LoaderManager>>()?;
    let loader_result = loaders.runs_by_id().load_one(LaxId::parse(id)?).await?;

    Ok(RunEventsFields::new(loader_result.expect_one()?.clone()))
  }

  pub async fn runs(
    &self,
    ctx: &Context<'_>,
    start: Option<DateScalar>,
    finish: Option<DateScalar>,
    // TODO: implement this?
    _exclude_conflicts: Option<DateScalar>,
  ) -> Result<Vec<RunEventsFields>, Error> {
    let loaders = ctx.data::<Arc<LoaderManager>>()?;
    Ok(
      loaders
        .event_runs_filtered
        .get(EventRunsLoaderFilter {
          start: start.map(|start| start.into()),
          finish: finish.map(|finish| finish.into()),
        })
        .await
        .load_one(self.model.id)
        .await?
        .unwrap_or_default()
        .iter()
        .map(|model| RunEventsFields::new(model.clone()))
        .collect(),
    )
  }

  pub async fn team_members(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<TeamMemberEventsFields>, Error> {
    let loader_result = load_one_by_model_id!(event_team_members, ctx, self)?;
    Ok(loader_result_to_many!(
      loader_result,
      TeamMemberEventsFields
    ))
  }
}

#[Object(guard = "EventPolicy::model_guard(EventAction::Read, self)")]
impl EventEventsFields {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(
    name = "admin_notes",
    guard = "EventPolicy::model_guard(EventAction::ReadAdminNotes, self)"
  )]
  async fn admin_notes(&self) -> Option<&str> {
    self.model.admin_notes.as_deref()
  }

  async fn author(&self) -> &Option<String> {
    &self.model.author
  }

  #[graphql(name = "can_play_concurrently")]
  async fn can_play_concurrently(&self) -> bool {
    self.model.can_play_concurrently
  }

  #[graphql(name = "con_mail_destination")]
  async fn con_mail_destination(&self) -> Option<&str> {
    self.model.con_mail_destination.as_deref()
  }

  #[graphql(name = "content_warnings")]
  async fn content_warnings(&self) -> Option<&str> {
    self.model.content_warnings.as_deref()
  }

  #[graphql(name = "created_at")]
  async fn created_at(&self) -> Result<Option<DateScalar>> {
    self.model.created_at.map(DateScalar::try_from).transpose()
  }

  async fn description(&self) -> Option<&str> {
    self.model.description.as_deref()
  }

  #[graphql(name = "description_html")]
  async fn description_html(&self, ctx: &Context<'_>) -> Result<String, Error> {
    let loaders = ctx.data::<Arc<LoaderManager>>()?;
    Ok(render_markdown(
      self.model.description.as_deref().unwrap_or_default(),
      &attached_images_by_filename(self.get_model(), loaders).await?,
    ))
  }

  async fn email(&self) -> &Option<String> {
    &self.model.email
  }

  async fn images(&self, ctx: &Context<'_>) -> Result<Vec<ActiveStorageAttachmentType>> {
    let blobs = ctx
      .data::<Arc<LoaderManager>>()?
      .event_attached_images
      .load_one(self.model.id)
      .await?
      .unwrap_or_default();

    Ok(
      blobs
        .into_iter()
        .map(ActiveStorageAttachmentType::new)
        .collect(),
    )
  }

  #[graphql(name = "length_seconds")]
  async fn length_seconds(&self) -> i32 {
    self.model.length_seconds
  }

  #[graphql(name = "my_rating")]
  async fn my_rating(&self, ctx: &Context<'_>) -> Result<Option<i32>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    let loaders = ctx.data::<Arc<LoaderManager>>()?;
    if let Some(user_con_profile) = query_data.user_con_profile() {
      let loader = loaders
        .event_user_con_profile_event_ratings
        .get(user_con_profile.id)
        .await;

      Ok(
        loader
          .load_one(self.model.id)
          .await?
          .and_then(|event_rating| event_rating.rating),
      )
    } else {
      Ok(None)
    }
  }

  async fn organization(&self) -> Option<&str> {
    self.model.organization.as_deref()
  }

  #[graphql(name = "participant_communications")]
  async fn participant_communications(&self) -> Option<&str> {
    self.model.participant_communications.as_deref()
  }

  #[graphql(name = "private_signup_list")]
  async fn private_signup_list(&self) -> bool {
    self.model.private_signup_list
  }

  #[graphql(name = "registration_policy")]
  async fn registration_policy(&self) -> Result<Option<RegistrationPolicyType>, serde_json::Error> {
    self
      .model
      .registration_policy
      .as_ref()
      .map(|policy| {
        serde_json::from_value::<RegistrationPolicy>(policy.clone()).map(RegistrationPolicyType)
      })
      .transpose()
  }

  #[graphql(name = "short_blurb")]
  async fn short_blurb(&self) -> Option<&str> {
    self.model.short_blurb.as_deref()
  }

  #[graphql(name = "short_blurb_html")]
  async fn short_blurb_html(&self, ctx: &Context<'_>) -> Result<String, Error> {
    let loaders = ctx.data::<Arc<LoaderManager>>()?;
    Ok(render_markdown(
      self.model.short_blurb.as_deref().unwrap_or_default(),
      &attached_images_by_filename(self.get_model(), loaders).await?,
    ))
  }

  async fn status(&self) -> &str {
    &self.model.status
  }

  async fn title(&self) -> &String {
    &self.model.title
  }

  async fn url(&self) -> Option<&str> {
    self.model.url.as_deref()
  }
}
