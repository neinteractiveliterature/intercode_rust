use std::{str::FromStr, sync::Arc};

use async_graphql::*;
use intercode_entities::{conventions, coupons, links::ConventionToOrders};
use intercode_graphql_core::{
  load_one_by_model_id, loader_result_to_many, model_backed_type, ModelPaginator,
};
use intercode_policies::AuthorizedFromQueryBuilder;
use intercode_query_builders::sort_input::SortInput;
use sea_orm::ModelTrait;

use crate::{
  objects::{CouponType, ProductType, StripeAccountType, TicketTypeType},
  policies::{CouponPolicy, OrderPolicy},
  query_builders::{
    CouponFiltersInput, CouponsQueryBuilder, OrderFiltersInput, OrdersQueryBuilder,
  },
};

use super::OrderStoreFields;

model_backed_type!(ConventionStoreFields, conventions::Model);

impl ConventionStoreFields {
  pub async fn coupons_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    per_page: Option<u64>,
    filters: Option<CouponFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<ModelPaginator<CouponType>, Error> {
    ModelPaginator::authorized_from_query_builder(
      &CouponsQueryBuilder::new(filters, sort),
      ctx,
      self.model.find_related(coupons::Entity),
      page,
      per_page,
      CouponPolicy,
    )
  }

  pub async fn orders_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    per_page: Option<u64>,
    filters: Option<OrderFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<ModelPaginator<OrderStoreFields>, Error> {
    ModelPaginator::authorized_from_query_builder(
      &OrdersQueryBuilder::new(filters, sort),
      ctx,
      self.model.find_linked(ConventionToOrders),
      page,
      per_page,
      OrderPolicy,
    )
  }
}

#[Object]
impl ConventionStoreFields {
  #[graphql(name = "coupons_paginated")]

  async fn products(&self, ctx: &Context<'_>) -> Result<Vec<ProductType>> {
    let loader_result = load_one_by_model_id!(convention_products, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, ProductType))
  }

  #[graphql(name = "stripe_account")]
  async fn stripe_account(&self, ctx: &Context<'_>) -> Result<Option<StripeAccountType>> {
    if let Some(id) = &self.model.stripe_account_id {
      let client = ctx.data::<Arc<stripe::Client>>()?;
      let acct = stripe::Account::retrieve(client, &stripe::AccountId::from_str(id)?, &[]).await?;
      Ok(Some(StripeAccountType::new(acct)))
    } else {
      Ok(None)
    }
  }

  #[graphql(name = "stripe_account_id")]
  async fn stripe_account_id(&self) -> Option<&str> {
    self.model.stripe_account_id.as_deref()
  }

  #[graphql(name = "stripe_account_ready_to_charge")]
  async fn stripe_account_ready_to_charge(&self) -> bool {
    self.model.stripe_account_ready_to_charge
  }

  #[graphql(name = "stripe_publishable_key")]
  async fn stripe_publishable_key(&self) -> Option<String> {
    std::env::var("STRIPE_PUBLISHABLE_KEY").ok()
  }

  #[graphql(name = "ticket_types")]
  async fn ticket_types(&self, ctx: &Context<'_>) -> Result<Vec<TicketTypeType>, Error> {
    let loader_result = load_one_by_model_id!(convention_ticket_types, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, TicketTypeType))
  }

  #[graphql(name = "tickets_available_for_purchase")]
  async fn tickets_available_for_purchase(&self) -> bool {
    self.model.tickets_available_for_purchase()
  }
}
