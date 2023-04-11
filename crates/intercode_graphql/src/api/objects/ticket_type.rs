use async_graphql::*;
use chrono::NaiveDateTime;
use intercode_entities::{conventions, tickets, user_con_profiles};
use intercode_policies::policies::{TicketAction, TicketPolicy};
use seawater::loaders::ExpectModels;

use crate::{model_backed_type, policy_guard::PolicyGuard, QueryData};

use super::{EventType, ModelBackedType, OrderEntryType, TicketTypeType, UserConProfileType};
model_backed_type!(TicketType, tickets::Model);

impl TicketType {
  fn policy_guard(
    &self,
    action: TicketAction,
  ) -> PolicyGuard<
    '_,
    TicketPolicy,
    (conventions::Model, user_con_profiles::Model, tickets::Model),
    tickets::Model,
  > {
    PolicyGuard::new(action, &self.model, move |model, ctx| {
      let model = model.clone();
      let ctx = ctx;
      let query_data = ctx.data::<QueryData>();

      Box::pin(async {
        let query_data = query_data?;
        let ticket_user_con_profile_loader = query_data.loaders().ticket_user_con_profile();
        let user_con_profile_convention_loader = query_data.loaders().user_con_profile_convention();

        let user_con_profile_result = ticket_user_con_profile_loader.load_one(model.id).await?;
        let user_con_profile = user_con_profile_result.expect_one()?;
        let convention_result = user_con_profile_convention_loader
          .load_one(user_con_profile.id)
          .await?;
        let convention = convention_result.expect_one()?;

        Ok((convention.clone(), user_con_profile.clone(), model))
      })
    })
  }
}

#[Object(name = "Ticket", guard = "self.policy_guard(TicketAction::Read)")]
impl TicketType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "created_at")]
  async fn created_at(&self) -> NaiveDateTime {
    self.model.created_at
  }

  #[graphql(name = "order_entry")]
  async fn order_entry(&self, ctx: &Context<'_>) -> Result<Option<OrderEntryType>> {
    Ok(
      ctx
        .data::<QueryData>()?
        .loaders()
        .ticket_order_entry()
        .load_one(self.model.id)
        .await?
        .try_one()
        .cloned()
        .map(OrderEntryType::new),
    )
  }

  #[graphql(name = "provided_by_event")]
  async fn provided_by_event(&self, ctx: &Context<'_>) -> Result<Option<EventType>> {
    let loader = ctx
      .data::<QueryData>()?
      .loaders()
      .ticket_provided_by_event();

    Ok(
      loader
        .load_one(self.model.id)
        .await?
        .try_one()
        .map(|event| EventType::new(event.clone())),
    )
  }

  #[graphql(name = "ticket_type")]
  async fn ticket_type(&self, ctx: &Context<'_>) -> Result<TicketTypeType> {
    let loader = ctx.data::<QueryData>()?.loaders().ticket_ticket_type();
    loader
      .load_one(self.model.id)
      .await?
      .expect_one()
      .map(|ticket_type| TicketTypeType::new(ticket_type.clone()))
  }

  #[graphql(name = "updated_at")]
  async fn updated_at(&self) -> NaiveDateTime {
    self.model.updated_at
  }

  #[graphql(name = "user_con_profile")]
  async fn user_con_profile(&self, ctx: &Context<'_>) -> Result<UserConProfileType> {
    let loader = ctx.data::<QueryData>()?.loaders().ticket_user_con_profile();
    loader
      .load_one(self.model.id)
      .await?
      .expect_one()
      .map(|user_con_profile| UserConProfileType::new(user_con_profile.clone()))
  }
}
