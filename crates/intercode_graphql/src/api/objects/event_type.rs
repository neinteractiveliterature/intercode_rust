use std::sync::Arc;

use crate::{
  api::merged_objects::{FormType, TicketType},
  QueryData,
};
use async_graphql::*;
use futures::StreamExt;
use intercode_entities::{events, RegistrationPolicy};
use intercode_forms::{
  form_response_implementation::attached_images_by_filename, partial_objects::EventFormsFields,
};
use intercode_graphql_core::{
  lax_id::LaxId, load_one_by_id, model_backed_type, objects::ActiveStorageAttachmentType,
  scalars::DateScalar, ModelBackedType,
};
use intercode_graphql_loaders::{filtered_event_runs_loader::EventRunsLoaderFilter, LoaderManager};
use intercode_liquid::render_markdown;
use intercode_policies::{
  policies::{EventAction, EventPolicy, MaximumEventProvidedTicketsOverridePolicy},
  AuthorizationInfo, ModelBackedTypeGuardablePolicy, Policy, ReadManageAction,
};
use seawater::loaders::{ExpectModel, ExpectModels};

use super::{
  ConventionType, EventCategoryType, MaximumEventProvidedTicketsOverrideType,
  RegistrationPolicyType, RunType, TeamMemberType,
};

model_backed_type!(EventApiFields, events::Model);

#[Object(guard = "EventPolicy::model_guard(EventAction::Read, self)")]
impl EventApiFields {
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

  async fn convention(&self, ctx: &Context<'_>) -> Result<ConventionType, Error> {
    let loader_result = load_one_by_id!(conventions_by_id, ctx, self.model.convention_id)?;
    Ok(ConventionType::new(loader_result.expect_one()?.clone()))
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
      &attached_images_by_filename(&self.model, loaders).await?,
    ))
  }

  async fn email(&self) -> &Option<String> {
    &self.model.email
  }

  async fn form(&self, ctx: &Context<'_>) -> Result<FormType, Error> {
    self.event_category(ctx).await?.event_form(ctx).await
  }

  #[graphql(name = "event_category")]
  async fn event_category(&self, ctx: &Context<'_>) -> Result<EventCategoryType, Error> {
    let loader = ctx.data::<Arc<LoaderManager>>()?.event_event_category();

    Ok(EventCategoryType::new(
      loader.load_one(self.model.id).await?.expect_one()?.clone(),
    ))
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

  #[graphql(name = "maximum_event_provided_tickets_overrides")]
  async fn maximum_event_provided_tickets_overrides(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<MaximumEventProvidedTicketsOverrideType>> {
    let loaders = ctx.data::<Arc<LoaderManager>>()?;
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let convention_result = loaders.event_convention().load_one(self.model.id).await?;
    let convention = convention_result.expect_one()?;
    let meptos_result = loaders
      .event_maximum_event_provided_tickets_overrides()
      .load_one(self.model.id)
      .await?;
    let meptos = meptos_result.expect_models()?;

    let meptos_stream = futures::stream::iter(meptos);
    let readable_meptos = meptos_stream.filter(|mepto| {
      let mepto = (*mepto).clone();
      async {
        MaximumEventProvidedTicketsOverridePolicy::action_permitted(
          authorization_info,
          &ReadManageAction::Read,
          &(convention.clone(), self.model.clone(), mepto),
        )
        .await
        .unwrap_or(false)
      }
    });
    let mepto_objects = readable_meptos
      .map(|mepto| MaximumEventProvidedTicketsOverrideType::new(mepto.clone()))
      .collect()
      .await;
    Ok(mepto_objects)
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

  #[graphql(name = "provided_tickets")]
  async fn provided_tickets(&self, ctx: &Context<'_>) -> Result<Vec<TicketType>> {
    let loader = ctx.data::<Arc<LoaderManager>>()?.event_provided_tickets();
    loader
      .load_one(self.model.id)
      .await?
      .expect_models()
      .map(|models| models.iter().cloned().map(TicketType::new).collect())
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

  async fn run(&self, ctx: &Context<'_>, id: ID) -> Result<RunType, Error> {
    let loaders = ctx.data::<Arc<LoaderManager>>()?;
    let loader_result = loaders.runs_by_id().load_one(LaxId::parse(id)?).await?;

    Ok(RunType::new(loader_result.expect_one()?.clone()))
  }

  async fn runs(
    &self,
    ctx: &Context<'_>,
    start: Option<DateScalar>,
    finish: Option<DateScalar>,
    #[graphql(name = "exclude_conflicts")] _exclude_conflicts: Option<DateScalar>,
  ) -> Result<Vec<RunType>, Error> {
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
        .map(|model| RunType::new(model.clone()))
        .collect(),
    )
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
      &attached_images_by_filename(&self.model, loaders).await?,
    ))
  }

  async fn status(&self) -> &str {
    &self.model.status
  }

  #[graphql(name = "team_members")]
  async fn team_members(&self, ctx: &Context<'_>) -> Result<Vec<TeamMemberType>, Error> {
    let loaders = ctx.data::<Arc<LoaderManager>>()?;
    Ok(
      loaders
        .event_team_members()
        .load_one(self.model.id)
        .await?
        .expect_models()?
        .iter()
        .map(|model| TeamMemberType::new(model.clone()))
        .collect(),
    )
  }

  async fn title(&self) -> &String {
    &self.model.title
  }

  async fn url(&self) -> Option<&str> {
    self.model.url.as_deref()
  }
}

#[derive(MergedObject)]
#[graphql(name = "Event")]
pub struct EventType(EventApiFields, EventFormsFields);

impl ModelBackedType for EventType {
  type Model = events::Model;

  fn new(model: Self::Model) -> Self {
    Self(
      EventApiFields::new(model.clone()),
      EventFormsFields::new(model),
    )
  }

  fn get_model(&self) -> &Self::Model {
    self.0.get_model()
  }

  fn into_model(self) -> Self::Model {
    self.0.into_model()
  }
}
