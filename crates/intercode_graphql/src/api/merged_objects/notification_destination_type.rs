use async_graphql::*;
use intercode_email::partial_objects::NotificationDestinationEmailFields;
use intercode_entities::notification_destinations;
use intercode_graphql_core::{model_backed_type, ModelBackedType};

use crate::{
  api::merged_objects::{StaffPositionType, UserConProfileType},
  merged_model_backed_type,
};

model_backed_type!(
  NotificationDestinationGlueFields,
  notification_destinations::Model
);

#[Object]
impl NotificationDestinationGlueFields {
  #[graphql(name = "staff_position")]
  async fn staff_position(&self, ctx: &Context<'_>) -> Result<Option<StaffPositionType>> {
    NotificationDestinationEmailFields::from_type(self.clone())
      .staff_position(ctx)
      .await
      .map(|res| res.map(StaffPositionType::new))
  }

  #[graphql(name = "user_con_profile")]
  async fn user_con_profile(&self, ctx: &Context<'_>) -> Result<Option<UserConProfileType>> {
    NotificationDestinationEmailFields::from_type(self.clone())
      .user_con_profile(ctx)
      .await
      .map(|res| res.map(UserConProfileType::new))
  }
}

merged_model_backed_type!(
  NotificationDestinationType,
  notification_destinations::Model,
  "NotificationDestination",
  NotificationDestinationEmailFields,
  NotificationDestinationGlueFields
);
