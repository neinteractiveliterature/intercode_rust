use async_graphql::*;
use intercode_entities::{orders, products, ticket_types, user_con_profiles};
use intercode_graphql_core::{load_one_by_id, query_data::QueryData};
use intercode_policies::{
  model_action_permitted::model_action_permitted, AuthorizationInfo, Policy, ReadManageAction,
};
use seawater::loaders::ExpectModel;
use std::borrow::Cow;

use crate::policies::{OrderAction, OrderPolicy, ProductPolicy, TicketTypePolicy};

pub struct AbilityStoreFields<'a> {
  authorization_info: Cow<'a, AuthorizationInfo>,
}

impl<'a> AbilityStoreFields<'a> {
  pub fn new(authorization_info: Cow<'a, AuthorizationInfo>) -> Self {
    Self { authorization_info }
  }
}

#[Object]
impl<'a> AbilityStoreFields<'a> {
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
}
