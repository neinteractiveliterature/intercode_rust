use intercode_entities::{
  events, links::EventToTeamMemberUserConProfiles, runs, user_con_profiles,
};
use intercode_graphql::{loaders::expect::ExpectModels, SchemaData};
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
use liquid::model::DateTime;
use sea_orm::{ModelTrait, PrimaryKeyToColumn};

use super::{
  preloaders::{EntityLinkPreloader, EntityRelationPreloader},
  utils::naive_date_time_to_liquid_date_time,
  DropError, RunDrop, UserConProfileDrop,
};

#[liquid_drop_struct]
pub struct EventDrop {
  event: events::Model,
  schema_data: SchemaData,
}

#[liquid_drop_impl]
impl EventDrop {
  pub fn new(event: events::Model, schema_data: SchemaData) -> Self {
    EventDrop { event, schema_data }
  }

  fn id(&self) -> i64 {
    self.event.id
  }

  fn created_at(&self) -> Option<DateTime> {
    self
      .event
      .created_at
      .and_then(naive_date_time_to_liquid_date_time)
  }

  fn title(&self) -> &str {
    self.event.title.as_str()
  }

  pub fn runs_preloader(
  ) -> EntityRelationPreloader<events::Entity, runs::Entity, events::PrimaryKey, Self, Vec<RunDrop>>
  {
    EntityRelationPreloader::new(
      events::PrimaryKey::Id.into_column(),
      |drop: &Self| drop.id(),
      |result| {
        let runs: &Vec<runs::Model> = result.expect_models()?;
        Ok(runs.iter().map(|run| RunDrop::new(run.clone())).collect())
      },
      |drop, value| drop.drop_cache.set_runs(value).map_err(|err| err.into()),
    )
  }

  async fn runs(&self) -> Result<Vec<RunDrop>, DropError> {
    Ok(
      self
        .event
        .find_related(runs::Entity)
        .all(self.schema_data.db.as_ref())
        .await?
        .into_iter()
        .map(RunDrop::new)
        .collect::<Vec<_>>(),
    )
  }

  pub fn team_member_user_con_profiles_preloader(
    schema_data: SchemaData,
  ) -> EntityLinkPreloader<
    events::Entity,
    EventToTeamMemberUserConProfiles,
    user_con_profiles::Entity,
    events::PrimaryKey,
    Self,
    Vec<UserConProfileDrop>,
  > {
    EntityLinkPreloader::new(
      events::PrimaryKey::Id.into_column(),
      EventToTeamMemberUserConProfiles,
      |drop: &Self| drop.id(),
      move |result| {
        let user_con_profiles = result.expect_models()?;
        Ok(
          user_con_profiles
            .iter()
            .map(|ucp| UserConProfileDrop::new(ucp.clone(), schema_data.clone()))
            .collect(),
        )
      },
      |drop, value| {
        drop
          .drop_cache
          .set_team_member_user_con_profiles(value)
          .map_err(|err| err.into())
      },
    )
  }

  pub async fn team_member_user_con_profiles(&self) -> Result<Vec<UserConProfileDrop>, DropError> {
    Ok(
      self
        .event
        .find_linked(EventToTeamMemberUserConProfiles)
        .all(self.schema_data.db.as_ref())
        .await?
        .into_iter()
        .map(|ucp| UserConProfileDrop::new(ucp, self.schema_data.clone()))
        .collect(),
    )
  }
}
