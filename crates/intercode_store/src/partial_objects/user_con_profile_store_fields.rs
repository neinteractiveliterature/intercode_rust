use async_graphql::*;
use async_trait::async_trait;
use intercode_entities::{
  events, maximum_event_provided_tickets_overrides, order_entries, orders, user_con_profiles,
};
use intercode_graphql_core::{
  load_one_by_model_id, loader_result_to_many, model_backed_type, query_data::QueryData,
  ModelBackedType,
};
use intercode_policies::{AuthorizationInfo, Policy, ReadManageAction};
use sea_orm::{sea_query::Expr, ColumnTrait, EntityTrait, QueryFilter};
use seawater::loaders::ExpectModels;

use crate::{
  order_summary_presenter::load_and_describe_order_summary_for_user_con_profile,
  policies::MaximumEventProvidedTicketsOverridePolicy,
};

model_backed_type!(UserConProfileStoreFields, user_con_profiles::Model);

#[async_trait]
pub trait UserConProfileStoreExtensions
where
  Self: ModelBackedType<Model = user_con_profiles::Model>,
{
  async fn current_pending_order<T: ModelBackedType<Model = orders::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<T>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    let pending_orders = orders::Entity::find()
      .filter(
        orders::Column::UserConProfileId
          .eq(self.get_model().id)
          .and(orders::Column::Status.eq("pending")),
      )
      .all(query_data.db())
      .await?;

    if pending_orders.is_empty() {
      Ok(None)
    } else if pending_orders.len() > 1 {
      // combine orders into one cart
      let (first, rest) = pending_orders.split_at(1);
      order_entries::Entity::update_many()
        .col_expr(
          order_entries::Column::OrderId,
          Expr::value(sea_orm::Value::BigInt(Some(first[0].id))),
        )
        .filter(
          order_entries::Column::OrderId
            .is_in(rest.iter().map(|order| order.id).collect::<Vec<i64>>()),
        )
        .exec(query_data.db())
        .await?;

      Ok(Some(T::new(first[0].to_owned())))
    } else {
      Ok(Some(T::new(pending_orders[0].to_owned())))
    }
  }

  async fn orders<T: ModelBackedType<Model = orders::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<T>> {
    let loader_result = load_one_by_model_id!(user_con_profile_orders, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, T))
  }
}

#[Object]
impl UserConProfileStoreFields {
  async fn id(&self) -> ID {
    ID(self.model.id.to_string())
  }

  #[graphql(name = "can_override_maximum_event_provided_tickets")]
  async fn can_override_maximum_event_provided_tickets(&self, ctx: &Context<'_>) -> Result<bool> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let query_data = ctx.data::<QueryData>()?;
    let Some(convention) = query_data.convention() else {
      return Ok(false);
    };

    Ok(
      MaximumEventProvidedTicketsOverridePolicy::action_permitted(
        authorization_info,
        &ReadManageAction::Manage,
        &(
          convention.clone(),
          events::Model {
            convention_id: convention.id,
            ..Default::default()
          },
          maximum_event_provided_tickets_overrides::Model {
            ..Default::default()
          },
        ),
      )
      .await?,
    )
  }

  #[graphql(name = "order_summary")]
  async fn order_summary(&self, ctx: &Context<'_>) -> Result<String> {
    let loader_result = load_one_by_model_id!(user_con_profile_orders, ctx, self)?;

    load_and_describe_order_summary_for_user_con_profile(loader_result.expect_models()?, ctx, true)
      .await
  }
}
