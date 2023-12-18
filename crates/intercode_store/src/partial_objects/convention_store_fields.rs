use std::{str::FromStr, sync::Arc};

use async_graphql::*;
use async_trait::async_trait;
use intercode_entities::{
  conventions, coupons, links::ConventionToOrders, orders, products, ticket_types,
};
use intercode_graphql_core::{
  lax_id::LaxId, load_one_by_model_id, loader_result_to_many, model_backed_type,
  query_data::QueryData, ModelBackedType, ModelPaginator,
};
use intercode_policies::{
  AuthorizationInfo, AuthorizedFromQueryBuilder, EntityPolicy, ReadManageAction,
};
use intercode_query_builders::sort_input::SortInput;
use sea_orm::{ColumnTrait, ModelTrait, QueryFilter};

use crate::{
  objects::StripeAccountType,
  policies::{CouponPolicy, OrderPolicy, ProductPolicy},
  query_builders::{
    CouponFiltersInput, CouponsQueryBuilder, OrderFiltersInput, OrdersQueryBuilder,
  },
};

model_backed_type!(ConventionStoreFields, conventions::Model);

#[async_trait]
pub trait ConventionStoreExtensions
where
  Self: ModelBackedType<Model = conventions::Model>,
{
  async fn coupons_paginated<T: ModelBackedType<Model = coupons::Model>>(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    per_page: Option<u64>,
    filters: Option<CouponFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<ModelPaginator<T>, Error> {
    ModelPaginator::authorized_from_query_builder(
      &CouponsQueryBuilder::new(filters, sort),
      ctx,
      self.get_model().find_related(coupons::Entity),
      page,
      per_page,
      CouponPolicy,
    )
  }

  async fn orders_paginated<T: ModelBackedType<Model = orders::Model>>(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    per_page: Option<u64>,
    filters: Option<OrderFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<ModelPaginator<T>, Error> {
    ModelPaginator::authorized_from_query_builder(
      &OrdersQueryBuilder::new(filters, sort),
      ctx,
      self.get_model().find_linked(ConventionToOrders),
      page,
      per_page,
      OrderPolicy,
    )
  }

  async fn product<T: ModelBackedType<Model = products::Model>>(
    &self,
    ctx: &Context<'_>,
    id: ID,
  ) -> Result<T> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let query_data = ctx.data::<QueryData>()?;
    Ok(T::new(
      ProductPolicy::accessible_to(authorization_info, &ReadManageAction::Read)
        .filter(products::Column::Id.eq(LaxId::parse(id)?))
        .one(query_data.db())
        .await?
        .ok_or_else(|| Error::new("Product not found"))?,
    ))
  }

  async fn products<T: ModelBackedType<Model = products::Model>>(
    &self,
    ctx: &Context<'_>,
    only_available: Option<bool>,
    only_ticket_providing: Option<bool>,
  ) -> Result<Vec<T>> {
    let loader_result = load_one_by_model_id!(convention_products, ctx, self)?;
    let all_products: Vec<T> = loader_result_to_many!(loader_result, T);
    let mut products_iter: Box<dyn Iterator<Item = T>> = Box::new(all_products.into_iter());

    if only_available.unwrap_or(false) {
      products_iter =
        Box::new(products_iter.filter(|product| product.get_model().available.unwrap_or(false)));
    }

    if only_ticket_providing.unwrap_or(false) {
      products_iter = Box::new(
        products_iter.filter(|product| product.get_model().provides_ticket_type_id.is_some()),
      );
    }

    Ok(products_iter.collect())
  }

  async fn ticket_types<T: ModelBackedType<Model = ticket_types::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<T>, Error> {
    let loader_result = load_one_by_model_id!(convention_ticket_types, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, T))
  }
}

#[Object]
impl ConventionStoreFields {
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

  #[graphql(name = "tickets_available_for_purchase")]
  async fn tickets_available_for_purchase(&self) -> bool {
    self.model.tickets_available_for_purchase()
  }
}
