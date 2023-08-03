use async_graphql::*;
use intercode_entities::{conventions, orders, products, ticket_types, tickets, user_con_profiles};
use intercode_graphql_core::{lax_id::LaxId, load_one_by_id, query_data::QueryData};
use intercode_graphql_loaders::LoaderManager;
use intercode_policies::{
  model_action_permitted::model_action_permitted, AuthorizationInfo, Policy, ReadManageAction,
};
use sea_orm::EntityTrait;
use seawater::loaders::ExpectModel;
use std::sync::Arc;

use crate::policies::{
  OrderAction, OrderPolicy, ProductPolicy, TicketAction, TicketPolicy, TicketTypePolicy,
};

pub struct AbilityStoreFields {
  authorization_info: Arc<AuthorizationInfo>,
}

impl AbilityStoreFields {
  pub fn new(authorization_info: Arc<AuthorizationInfo>) -> Self {
    Self { authorization_info }
  }

  async fn get_ticket_policy_model(
    &self,
    ctx: &Context<'_>,
    ticket_id: ID,
  ) -> Result<(conventions::Model, user_con_profiles::Model, tickets::Model), Error> {
    let query_data = ctx.data::<QueryData>()?;
    let loaders = ctx.data::<Arc<LoaderManager>>()?;
    let ticket = tickets::Entity::find_by_id(LaxId::parse(ticket_id)?)
      .one(query_data.db())
      .await?
      .ok_or_else(|| Error::new("Ticket not found"))?;

    let user_con_profile_result = loaders
      .ticket_user_con_profile()
      .load_one(ticket.id)
      .await?;
    let user_con_profile = user_con_profile_result.expect_one()?;

    let convention_result = loaders
      .user_con_profile_convention()
      .load_one(user_con_profile.id)
      .await?;
    let convention = convention_result.expect_one()?;

    Ok((convention.clone(), user_con_profile.clone(), ticket))
  }
}

#[Object]
impl AbilityStoreFields {
  #[graphql(name = "can_read_orders")]
  async fn can_read_orders(&self, ctx: &Context<'_>) -> Result<bool> {
    let authorization_info = self.authorization_info.as_ref();
    let convention = ctx.data::<QueryData>()?.convention();
    let Some(convention)= convention else {
      return Ok(false);
    };

    Ok(
      OrderPolicy::action_permitted(
        authorization_info,
        &OrderAction::Read,
        &(
          convention.clone(),
          user_con_profiles::Model {
            convention_id: convention.id,
            ..Default::default()
          },
          orders::Model {
            ..Default::default()
          },
          vec![],
        ),
      )
      .await?,
    )
  }

  #[graphql(name = "can_create_orders")]
  async fn can_create_orders(&self, ctx: &Context<'_>) -> Result<bool> {
    let authorization_info = self.authorization_info.as_ref();
    let convention = ctx.data::<QueryData>()?.convention();
    let Some(convention)= convention else {
      return Ok(false);
    };

    Ok(
      OrderPolicy::action_permitted(
        authorization_info,
        &OrderAction::Manage,
        &(
          convention.clone(),
          user_con_profiles::Model {
            convention_id: convention.id,
            ..Default::default()
          },
          orders::Model {
            ..Default::default()
          },
          vec![],
        ),
      )
      .await?,
    )
  }

  #[graphql(name = "can_update_orders")]
  async fn can_update_orders(&self, ctx: &Context<'_>) -> Result<bool> {
    let authorization_info = self.authorization_info.as_ref();
    let convention = ctx.data::<QueryData>()?.convention();
    let Some(convention)= convention else {
      return Ok(false);
    };

    Ok(
      OrderPolicy::action_permitted(
        authorization_info,
        &OrderAction::Manage,
        &(
          convention.clone(),
          user_con_profiles::Model {
            convention_id: convention.id,
            ..Default::default()
          },
          orders::Model {
            ..Default::default()
          },
          vec![],
        ),
      )
      .await?,
    )
  }

  #[graphql(name = "can_update_products")]
  async fn can_update_products(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    model_action_permitted(
      self.authorization_info.as_ref(),
      ProductPolicy,
      ctx,
      &ReadManageAction::Manage,
      |ctx| {
        Ok(Some(products::Model {
          convention_id: ctx.data::<QueryData>()?.convention().map(|con| con.id),
          ..Default::default()
        }))
      },
    )
    .await
  }

  #[graphql(name = "can_manage_ticket_types")]
  async fn can_manage_ticket_types(&self, ctx: &Context<'_>) -> Result<bool> {
    let authorization_info = self.authorization_info.as_ref();
    let convention = ctx.data::<QueryData>()?.convention();
    let Some(convention)= convention else {
      return Ok(false);
    };
    let single_event_loader_result = load_one_by_id!(convention_single_event, ctx, convention.id)?;
    let single_event = single_event_loader_result.try_one();

    Ok(
      TicketTypePolicy::action_permitted(
        authorization_info,
        &ReadManageAction::Manage,
        &(
          convention.clone(),
          single_event.cloned(),
          ticket_types::Model {
            convention_id: Some(convention.id),
            ..Default::default()
          },
        ),
      )
      .await?,
    )
  }

  #[graphql(name = "can_create_tickets")]
  async fn can_create_tickets(&self, ctx: &Context<'_>) -> Result<bool> {
    let convention = ctx.data::<QueryData>()?.convention();

    if let Some(convention) = convention {
      let user_con_profile = user_con_profiles::Model {
        convention_id: convention.id,
        ..Default::default()
      };
      let ticket = tickets::Model {
        ..Default::default()
      };

      model_action_permitted(
        &self.authorization_info,
        TicketPolicy,
        ctx,
        &TicketAction::Manage,
        |_ctx| Ok(Some((convention.clone(), user_con_profile, ticket))),
      )
      .await
    } else {
      Ok(false)
    }
  }

  #[graphql(name = "can_delete_ticket")]
  async fn can_delete_ticket(&self, ctx: &Context<'_>, ticket_id: ID) -> Result<bool> {
    Ok(
      TicketPolicy::action_permitted(
        &self.authorization_info,
        &TicketAction::Manage,
        &(self.get_ticket_policy_model(ctx, ticket_id).await?),
      )
      .await?,
    )
  }

  #[graphql(name = "can_update_ticket")]
  async fn can_update_ticket(&self, ctx: &Context<'_>, ticket_id: ID) -> Result<bool> {
    Ok(
      TicketPolicy::action_permitted(
        &self.authorization_info,
        &TicketAction::Manage,
        &(self.get_ticket_policy_model(ctx, ticket_id).await?),
      )
      .await?,
    )
  }
}
