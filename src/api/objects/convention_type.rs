use crate::{
  conventions, staff_positions_user_con_profiles, team_members, user_con_profiles, SchemaData,
};
use async_graphql::*;
use sea_orm::{ColumnTrait, ModelTrait, QueryFilter};

use super::{ModelBackedType, UserConProfileType};

use crate::model_backed_type;
model_backed_type!(ConventionType, conventions::Model);

#[Object]
impl ConventionType {
  async fn id(&self) -> ID {
    ID(self.model.id.to_string())
  }

  async fn name(&self) -> &Option<String> {
    &self.model.name
  }

  #[graphql(name = "bio_eligible_user_con_profiles")]
  async fn bio_eligible_user_con_profiles(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<UserConProfileType>, Error> {
    let db = &ctx.data::<SchemaData>()?.db;

    let profiles: Vec<UserConProfileType> = self
      .model
      .find_related(user_con_profiles::Entity)
      .left_join(staff_positions_user_con_profiles::Entity)
      .left_join(team_members::Entity)
      .filter(
        staff_positions_user_con_profiles::Column::StaffPositionId
          .is_not_null()
          .or(team_members::Column::Id.is_not_null()),
      )
      .all(db.as_ref())
      .await?
      .iter()
      .map(|model| UserConProfileType::new(model.to_owned()))
      .collect::<Vec<UserConProfileType>>();

    Ok(profiles)
  }
}
