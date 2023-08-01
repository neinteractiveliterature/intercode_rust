use async_graphql::*;
use intercode_entities::notification_destinations;
use intercode_graphql_core::{
  load_one_by_model_id, loader_result_to_optional_single, model_backed_type,
};

use crate::api::merged_objects::UserConProfileType;

use super::StaffPositionType;
model_backed_type!(
  NotificationDestinationType,
  notification_destinations::Model
);

#[Object(name = "NotificationDestination")]
impl NotificationDestinationType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "staff_position")]
  async fn staff_position(&self, ctx: &Context<'_>) -> Result<Option<StaffPositionType>> {
    let loader_result = load_one_by_model_id!(notification_destination_staff_position, ctx, self)?;
    Ok(loader_result_to_optional_single!(
      loader_result,
      StaffPositionType
    ))
  }

  #[graphql(name = "user_con_profile")]
  async fn user_con_profile(&self, ctx: &Context<'_>) -> Result<Option<UserConProfileType>> {
    let loader_result =
      load_one_by_model_id!(notification_destination_user_con_profile, ctx, self)?;
    Ok(loader_result_to_optional_single!(
      loader_result,
      UserConProfileType
    ))
  }
}
