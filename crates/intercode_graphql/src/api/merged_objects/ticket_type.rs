use async_graphql::*;
use intercode_entities::tickets;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_policies::{
  policies::{TicketAction, TicketPolicy},
  ModelBackedTypeGuardablePolicy,
};
use intercode_store::partial_objects::TicketStoreFields;

use crate::api::objects::UserConProfileType;

use super::{EventType, OrderEntryType};

model_backed_type!(TicketGlueFields, tickets::Model);

#[Object(guard = "TicketPolicy::model_guard(TicketAction::Read, self)")]
impl TicketGlueFields {
  #[graphql(name = "order_entry")]
  async fn order_entry(&self, ctx: &Context<'_>) -> Result<Option<OrderEntryType>> {
    TicketStoreFields::from_type(self.clone())
      .order_entry(ctx)
      .await
      .map(|res| res.map(OrderEntryType::from_type))
  }

  #[graphql(name = "provided_by_event")]
  async fn provided_by_event(&self, ctx: &Context<'_>) -> Result<Option<EventType>> {
    TicketStoreFields::from_type(self.clone())
      .provided_by_event(ctx)
      .await
      .map(|res| res.map(EventType::new))
  }

  #[graphql(name = "user_con_profile")]
  async fn user_con_profile(&self, ctx: &Context<'_>) -> Result<UserConProfileType> {
    TicketStoreFields::from_type(self.clone())
      .user_con_profile(ctx)
      .await
      .map(UserConProfileType::from_type)
  }
}

#[derive(MergedObject)]
#[graphql(name = "Ticket")]
pub struct TicketType(TicketStoreFields, TicketGlueFields);

impl ModelBackedType for TicketType {
  type Model = tickets::Model;

  fn new(model: Self::Model) -> Self {
    Self(
      TicketStoreFields::new(model.clone()),
      TicketGlueFields::new(model),
    )
  }

  fn get_model(&self) -> &Self::Model {
    self.0.get_model()
  }

  fn into_model(self) -> Self::Model {
    self.0.into_model()
  }
}
