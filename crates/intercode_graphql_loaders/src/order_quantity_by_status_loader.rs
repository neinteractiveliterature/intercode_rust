use std::collections::HashMap;

use async_graphql::{dataloader::Loader, EnumType, Object};
use async_trait::async_trait;
use intercode_entities::{order_entries, orders};
use intercode_graphql_core::enums::OrderStatus;
use itertools::Itertools;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};
use seawater::ConnectionWrapper;

#[derive(Debug, Clone)]
pub struct OrderQuantityByStatusType {
  status: String,
  quantity: i64,
}

impl OrderQuantityByStatusType {
  pub fn new(status: String, quantity: i64) -> Self {
    Self { status, quantity }
  }
}

#[Object(name = "OrderQuantityByStatus")]
impl OrderQuantityByStatusType {
  pub async fn status(&self) -> &str {
    &self.status
  }

  pub async fn quantity(&self) -> i64 {
    self.quantity
  }
}

pub enum OrderQuantityByStatusLoaderEntity {
  Product,
  ProductVariant,
}

pub struct OrderQuantityByStatusLoader {
  db: ConnectionWrapper,
  entity: OrderQuantityByStatusLoaderEntity,
}

impl OrderQuantityByStatusLoader {
  pub fn new(db: ConnectionWrapper, entity: OrderQuantityByStatusLoaderEntity) -> Self {
    OrderQuantityByStatusLoader { db, entity }
  }
}

#[async_trait]
impl Loader<i64> for OrderQuantityByStatusLoader {
  type Value = Vec<OrderQuantityByStatusType>;
  type Error = async_graphql::Error;

  async fn load(&self, keys: &[i64]) -> Result<HashMap<i64, Self::Value>, Self::Error> {
    let scope = order_entries::Entity::find();

    let scope = match self.entity {
      OrderQuantityByStatusLoaderEntity::Product => scope
        .filter(order_entries::Column::ProductId.is_in(keys.iter().copied()))
        .filter(order_entries::Column::ProductVariantId.is_null()),
      OrderQuantityByStatusLoaderEntity::ProductVariant => {
        scope.filter(order_entries::Column::ProductVariantId.is_in(keys.iter().copied()))
      }
    };

    let key_column = match self.entity {
      OrderQuantityByStatusLoaderEntity::Product => order_entries::Column::ProductId,
      OrderQuantityByStatusLoaderEntity::ProductVariant => order_entries::Column::ProductVariantId,
    };

    let mut results = scope
      .inner_join(orders::Entity)
      .select_only()
      .column_as(key_column, "key")
      .column_as(orders::Column::Status, "status")
      .column_as(order_entries::Column::Quantity.sum(), "quantity")
      .group_by(key_column)
      .group_by(orders::Column::Status)
      .into_tuple()
      .all(&self.db)
      .await?
      .into_iter()
      .map(|(key, status, quantity): (i64, String, i64)| {
        (key, OrderQuantityByStatusType::new(status, quantity))
      })
      .into_group_map();

    for key in keys {
      let entry = results.entry(*key);
      entry.or_insert_with(|| {
        OrderStatus::items()
          .iter()
          .filter_map(|status| {
            if status.value == OrderStatus::Pending {
              None
            } else {
              Some(OrderQuantityByStatusType::new(status.name.to_string(), 0))
            }
          })
          .collect()
      });
    }

    Ok(results)
  }
}
