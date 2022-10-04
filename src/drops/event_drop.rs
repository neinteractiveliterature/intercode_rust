use intercode_entities::{
  event_categories, events, links::EventToTeamMemberUserConProfiles, runs, user_con_profiles,
};
use intercode_graphql::{loaders::expect::ExpectModels, SchemaData};
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
use liquid::model::DateTime;
use sea_orm::{ModelTrait, PrimaryKeyToColumn};

use super::{
  preloaders::{EntityLinkPreloader, EntityRelationPreloader, Preloader},
  utils::naive_date_time_to_liquid_date_time,
  DropError, EventCategoryDrop, RunDrop, UserConProfileDrop,
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

  pub fn event_category_preloader() -> EntityRelationPreloader<
    events::Entity,
    event_categories::Entity,
    events::PrimaryKey,
    Self,
    EventCategoryDrop,
  > {
    EntityRelationPreloader::new(
      events::PrimaryKey::Id.into_column(),
      |drop: &Self| drop.id(),
      |result| {
        result
          .expect_one()
          .map(|event_category: &event_categories::Model| {
            EventCategoryDrop::new(event_category.clone())
          })
          .map_err(|err| err.into())
      },
      |cache| &cache.event_category,
    )
  }

  pub async fn event_category(&self) -> Result<EventCategoryDrop, DropError> {
    EventDrop::event_category_preloader()
      .load_single(&self.schema_data.db, self)
      .await
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
      |cache| &cache.runs,
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
      |cache| &cache.team_member_user_con_profiles,
    )
  }

  pub async fn team_member_user_con_profiles(&self) -> Result<&Vec<UserConProfileDrop>, DropError> {
    EventDrop::team_member_user_con_profiles_preloader(self.schema_data.clone())
      .load_single(&self.schema_data.db, self)
      .await?;
    Ok(
      self
        .drop_cache
        .team_member_user_con_profiles
        .get()
        .unwrap()
        .get_inner()
        .unwrap(),
    )
  }
}
