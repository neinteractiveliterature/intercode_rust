use async_graphql::*;
use intercode_entities::{notification_destinations, staff_positions, user_con_profiles};
use intercode_graphql_core::{load_one_by_model_id, model_backed_type};
use seawater::loaders::ExpectModel;

model_backed_type!(
  NotificationDestinationEmailFields,
  notification_destinations::Model
);

impl NotificationDestinationEmailFields {
  pub async fn staff_position(&self, ctx: &Context<'_>) -> Result<Option<staff_positions::Model>> {
    let loader_result = load_one_by_model_id!(notification_destination_staff_position, ctx, self)?;
    Ok(loader_result.try_one().cloned())
  }

  pub async fn user_con_profile(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<user_con_profiles::Model>> {
    let loader_result =
      load_one_by_model_id!(notification_destination_user_con_profile, ctx, self)?;
    Ok(loader_result.try_one().cloned())
  }
}

#[Object]
impl NotificationDestinationEmailFields {
  async fn id(&self) -> ID {
    self.model.id.into()
  }
}
