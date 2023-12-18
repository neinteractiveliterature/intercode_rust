use crate::policies::MaximumEventProvidedTicketsOverridePolicy;
use async_graphql::*;
use async_trait::async_trait;
use intercode_entities::{events, maximum_event_provided_tickets_overrides, ticket_types};
use intercode_graphql_core::{
  load_one_by_model_id, loader_result_to_required_single, model_backed_type, ModelBackedType,
};
use intercode_policies::{ModelBackedTypeGuardablePolicy, ReadManageAction};

model_backed_type!(
  MaximumEventProvidedTicketsOverrideStoreFields,
  maximum_event_provided_tickets_overrides::Model
);

#[async_trait]
pub trait MaximumEventProvidedTicketsOverrideStoreExtensions
where
  Self: ModelBackedType<Model = maximum_event_provided_tickets_overrides::Model>,
{
  async fn event<T: ModelBackedType<Model = events::Model>>(&self, ctx: &Context<'_>) -> Result<T> {
    let loader_result =
      load_one_by_model_id!(maximum_event_provided_tickets_override_event, ctx, self)?;
    Ok(loader_result_to_required_single!(loader_result, T))
  }

  async fn ticket_type<T: ModelBackedType<Model = ticket_types::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<T> {
    let loader_result = load_one_by_model_id!(
      maximum_event_provided_tickets_override_ticket_type,
      ctx,
      self
    )?;
    Ok(loader_result_to_required_single!(loader_result, T))
  }
}

#[Object(
  guard = "MaximumEventProvidedTicketsOverridePolicy::model_guard(ReadManageAction::Read, self)"
)]
impl MaximumEventProvidedTicketsOverrideStoreFields {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "override_value")]
  async fn override_value(&self) -> i32 {
    self.model.override_value
  }
}
