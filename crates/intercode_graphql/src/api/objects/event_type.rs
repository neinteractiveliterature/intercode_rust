use crate::{
  api::scalars::DateScalar, loaders::event_runs_loader::EventRunsLoaderFilter, QueryData,
};
use async_graphql::*;
use intercode_entities::{events, RegistrationPolicy};
use seawater::loaders::{ExpectModel, ExpectModels};

use super::{
  ConventionType, EventCategoryType, ModelBackedType, RegistrationPolicyType, RunType,
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

  async fn convention(&self, ctx: &Context<'_>) -> Result<Option<ConventionType>, Error> {
    let loader = &ctx.data::<QueryData>()?.loaders.conventions_by_id;

    if let Some(convention_id) = self.model.convention_id {
      let model = loader.load_one(convention_id).await?.expect_model()?;
      Ok(Some(ConventionType::new(model)))
    } else {
      Ok(None)
    }
  }

  async fn email(&self) -> &Option<String> {
    &self.model.email
  }

  #[graphql(name = "event_category")]
  async fn event_category(&self, ctx: &Context<'_>) -> Result<EventCategoryType, Error> {
    let loader = &ctx.data::<QueryData>()?.loaders.event_event_category;

    Ok(EventCategoryType::new(
      loader.load_one(self.model.id).await?.expect_one()?.clone(),
    ))
  }

  #[graphql(name = "length_seconds")]
  async fn length_seconds(&self) -> i32 {
    self.model.length_seconds
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
        .event_runs_loader_manager
        .with_filter(EventRunsLoaderFilter {
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

  #[graphql(name = "team_members")]
  async fn team_members(&self, ctx: &Context<'_>) -> Result<Vec<TeamMemberType>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    Ok(
      query_data
        .loaders
        .event_team_members
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
