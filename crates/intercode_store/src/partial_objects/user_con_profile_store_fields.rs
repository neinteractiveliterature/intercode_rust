use async_graphql::*;
use intercode_entities::{order_entries, orders, user_con_profiles};
use intercode_graphql_core::{
  load_one_by_model_id, model_backed_type, query_data::QueryData, ModelBackedType,
};
use sea_orm::{sea_query::Expr, ColumnTrait, EntityTrait, QueryFilter};
use seawater::loaders::ExpectModels;

use crate::order_summary_presenter::load_and_describe_order_summary_for_user_con_profile;

use super::OrderStoreFields;

model_backed_type!(UserConProfileStoreFields, user_con_profiles::Model);

impl UserConProfileStoreFields {
  pub async fn current_pending_order(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<OrderStoreFields>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    let pending_orders = orders::Entity::find()
      .filter(
        orders::Column::UserConProfileId
          .eq(self.model.id)
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

      Ok(Some(OrderStoreFields::new(first[0].to_owned())))
    } else {
      Ok(Some(OrderStoreFields::new(pending_orders[0].to_owned())))
    }
  }
}

#[Object]
impl UserConProfileStoreFields {
  async fn id(&self) -> ID {
    ID(self.model.id.to_string())
  }

  #[graphql(name = "order_summary")]
  async fn order_summary(&self, ctx: &Context<'_>) -> Result<String> {
    let loader_result = load_one_by_model_id!(user_con_profile_orders, ctx, self)?;

    load_and_describe_order_summary_for_user_con_profile(loader_result.expect_models()?, ctx, true)
      .await
  }
}
