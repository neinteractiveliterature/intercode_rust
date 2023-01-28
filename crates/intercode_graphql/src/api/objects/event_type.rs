use std::sync::Arc;

use crate::{
  api::{
    interfaces::FormResponseImplementation,
    scalars::{DateScalar, JsonScalar},
  },
  loaders::filtered_event_runs_loader::EventRunsLoaderFilter,
  presenters::form_response_presenter::attached_images_by_filename,
  QueryData,
};
use async_graphql::*;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use intercode_entities::{
  events, forms, model_ext::form_item_permissions::FormItemRole, RegistrationPolicy,
};
use intercode_liquid::render_markdown;
use intercode_policies::{policies::EventPolicy, AuthorizationInfo, FormResponsePolicy};
use seawater::loaders::{ExpectModel, ExpectModels};

use super::{
  ConventionType, EventCategoryType, FormType, ModelBackedType, RegistrationPolicyType, RunType,
  TeamMemberType,
};
use crate::model_backed_type;
model_backed_type!(EventType, events::Model);

#[Object(name = "Event")]
impl EventType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn author(&self) -> &Option<String> {
    &self.model.author
  }

  #[graphql(name = "can_play_concurrently")]
  async fn can_play_concurrently(&self) -> bool {
    self.model.can_play_concurrently
  }

  async fn convention(&self, ctx: &Context<'_>) -> Result<ConventionType, Error> {
    let loader = &ctx.data::<QueryData>()?.loaders.conventions_by_id;

    let model = loader
      .load_one(self.model.convention_id)
      .await?
      .expect_model()?;
    Ok(ConventionType::new(model))
  }

  #[graphql(name = "created_at")]
  async fn created_at(&self) -> Option<NaiveDateTime> {
    self.model.created_at
  }

  async fn email(&self) -> &Option<String> {
    &self.model.email
  }

  async fn form(&self, ctx: &Context<'_>) -> Result<FormType, Error> {
    self.event_category(ctx).await?.event_form(ctx).await
  }

  #[graphql(name = "event_category")]
  async fn event_category(&self, ctx: &Context<'_>) -> Result<EventCategoryType, Error> {
    let loader = &ctx.data::<QueryData>()?.loaders.event_event_category();

    Ok(EventCategoryType::new(
      loader.load_one(self.model.id).await?.expect_one()?.clone(),
    ))
  }

  #[graphql(name = "form_response_attrs_json")]
  async fn form_response_attrs_json(
    &self,
    ctx: &Context<'_>,
    item_identifiers: Option<Vec<String>>,
  ) -> Result<JsonScalar, Error> {
    <Self as FormResponseImplementation<events::Model>>::form_response_attrs_json(
      self,
      ctx,
      item_identifiers,
    )
    .await
  }

  #[graphql(name = "form_response_attrs_json_with_rendered_markdown")]
  async fn form_response_attrs_json_with_rendered_markdown(
    &self,
    ctx: &Context<'_>,
    item_identifiers: Option<Vec<String>>,
  ) -> Result<JsonScalar, Error> {
    <Self as FormResponseImplementation<events::Model>>::form_response_attrs_json_with_rendered_markdown(
      self,
      ctx,
      item_identifiers,
    )
    .await
  }

  #[graphql(name = "length_seconds")]
  async fn length_seconds(&self) -> i32 {
    self.model.length_seconds
  }

  #[graphql(name = "my_rating")]
  async fn my_rating(&self, ctx: &Context<'_>) -> Result<Option<i32>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    if let Some(user_con_profile) = query_data.user_con_profile.as_ref().as_ref() {
      let loader = query_data
        .loaders
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

  async fn run(&self, ctx: &Context<'_>, id: ID) -> Result<RunType, Error> {
    let query_data = ctx.data::<QueryData>()?;
    let run = query_data
      .loaders
      .runs_by_id
      .load_one(id.parse()?)
      .await?
      .expect_model()?;

    Ok(RunType::new(run))
  }

  async fn runs(
    &self,
    ctx: &Context<'_>,
    start: Option<DateScalar>,
    finish: Option<DateScalar>,
    #[graphql(name = "exclude_conflicts")] _exclude_conflicts: Option<DateScalar>,
  ) -> Result<Vec<RunType>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    Ok(
      query_data
        .loaders
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

  #[graphql(name = "short_blurb_html")]
  async fn short_blurb_html(&self, ctx: &Context<'_>) -> Result<String, Error> {
    let query_data = ctx.data::<QueryData>()?;
    Ok(render_markdown(
      self.model.short_blurb.as_deref().unwrap_or_default(),
      &attached_images_by_filename(&self.model, query_data).await?,
    ))
  }

  #[graphql(name = "team_members")]
  async fn team_members(&self, ctx: &Context<'_>) -> Result<Vec<TeamMemberType>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    Ok(
      query_data
        .loaders
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
}

#[async_trait]
impl FormResponseImplementation<events::Model> for EventType {
  async fn get_form(&self, ctx: &Context<'_>) -> Result<forms::Model, Error> {
    let query_data = ctx.data::<QueryData>()?;
    let event_category_result = query_data
      .loaders
      .event_event_category()
      .load_one(self.model.id)
      .await?;
    let event_category = event_category_result.expect_one()?;

    Ok(
      query_data
        .loaders
        .event_category_event_form
        .load_one(event_category.id)
        .await?
        .expect_one()?
        .clone(),
    )
  }

  async fn get_team_member_name(&self, ctx: &Context<'_>) -> Result<String, Error> {
    let query_data = ctx.data::<QueryData>()?;
    let event_category_result = query_data
      .loaders
      .event_event_category()
      .load_one(self.model.id)
      .await?;
    let event_category = event_category_result.expect_one()?;

    Ok(event_category.team_member_name.clone())
  }

  async fn get_viewer_role(&self, ctx: &Context<'_>) -> Result<FormItemRole, Error> {
    let authorization_info = ctx.data::<Arc<AuthorizationInfo>>()?;
    Ok(EventPolicy::form_item_viewer_role(authorization_info.as_ref(), &self.model).await)
  }
}
