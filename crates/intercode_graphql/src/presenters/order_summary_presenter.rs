use std::sync::Arc;

use async_graphql::{Context, Error};
use futures::{future::try_join_all, try_join};
use intercode_entities::{order_entries, orders, product_variants, products};
use intercode_graphql_loaders::LoaderManager;
use intercode_inflector::inflector::string::pluralize;
use seawater::loaders::{ExpectModel, ExpectModels};

pub fn describe_order_entry(
  order_entry: &order_entries::Model,
  product: &products::Model,
  product_variant: Option<&product_variants::Model>,
  always_show_quantity: bool,
) -> String {
  let mut parts: Vec<String> = Vec::with_capacity(3);

  if always_show_quantity || order_entry.quantity.filter(|q| *q > 1).is_some() {
    parts.push(order_entry.quantity.unwrap_or(1).to_string());
  }

  parts.push(if order_entry.quantity.unwrap_or(1) > 1 {
    pluralize::to_plural(&product.name.clone().unwrap_or_default())
  } else {
    product.name.clone().unwrap_or_default()
  });

  if let Some(product_variant) = product_variant {
    parts.push(format!(
      "({})",
      product_variant.name.clone().unwrap_or_default()
    ));
  }

  parts.join(" ")
}

pub async fn load_and_describe_order_entry(
  order_entry: &order_entries::Model,
  ctx: &Context<'_>,
  always_show_quantity: bool,
) -> Result<String, Error> {
  let loaders = ctx.data::<Arc<LoaderManager>>()?;

  let (product_result, product_variant_result) = try_join!(
    loaders.order_entry_product().load_one(order_entry.id),
    loaders
      .order_entry_product_variant()
      .load_one(order_entry.id),
  )?;

  let product = product_result.expect_one()?;
  let product_variant = product_variant_result.try_one();

  Ok(describe_order_entry(
    order_entry,
    product,
    product_variant,
    always_show_quantity,
  ))
}

pub async fn load_and_describe_order(
  order: &orders::Model,
  ctx: &Context<'_>,
  always_show_quantity: bool,
) -> Result<String, Error> {
  let loaders = ctx.data::<Arc<LoaderManager>>()?;
  let order_entries_result = loaders.order_order_entries().load_one(order.id).await?;
  let order_entries = order_entries_result.expect_models()?;

  let entry_summaries = try_join_all(
    order_entries
      .iter()
      .map(|entry| load_and_describe_order_entry(entry, ctx, always_show_quantity)),
  )
  .await?;

  Ok(
    entry_summaries
      .into_iter()
      .map(|entry_summary| format!("{} ({})", entry_summary, order.status))
      .collect::<Vec<_>>()
      .join(", "),
  )
}

pub async fn load_and_describe_order_summary_for_user_con_profile(
  orders: impl IntoIterator<Item = &orders::Model>,
  ctx: &Context<'_>,
  always_show_quantity: bool,
) -> Result<String, Error> {
  let futures = orders
    .into_iter()
    .filter(|order| order.status != "pending" && order.status != "cancelled")
    .map(|order| load_and_describe_order(order, ctx, always_show_quantity));

  let order_summaries = try_join_all(futures).await?;

  Ok(order_summaries.join(", "))
}
