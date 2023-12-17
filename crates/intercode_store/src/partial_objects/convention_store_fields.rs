use std::{str::FromStr, sync::Arc};

use async_graphql::*;
use intercode_entities::{conventions, coupons, links::ConventionToOrders, products};
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

use super::{CouponStoreFields, OrderStoreFields, ProductStoreFields, TicketTypeStoreFields};

model_backed_type!(ConventionStoreFields, conventions::Model);

impl ConventionStoreFields {
  pub async fn coupons_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    per_page: Option<u64>,
    filters: Option<CouponFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<ModelPaginator<CouponStoreFields>, Error> {
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

  /// Finds a product by ID in this convention. If there is no product with that ID in this
  /// convention, errors out.
  pub async fn product(&self, ctx: &Context<'_>, id: ID) -> Result<ProductStoreFields> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let query_data = ctx.data::<QueryData>()?;
    Ok(ProductStoreFields::new(
      ProductPolicy::accessible_to(authorization_info, &ReadManageAction::Read)
        .filter(products::Column::Id.eq(LaxId::parse(id)?))
        .one(query_data.db())
        .await?
        .ok_or_else(|| Error::new("Product not found"))?,
    ))
  }

  pub async fn products(
    &self,
    ctx: &Context<'_>,
    only_available: Option<bool>,
    only_ticket_providing: Option<bool>,
  ) -> Result<Vec<ProductStoreFields>> {
    let loader_result = load_one_by_model_id!(convention_products, ctx, self)?;
    let all_products: Vec<ProductStoreFields> =
      loader_result_to_many!(loader_result, ProductStoreFields);
    let mut products_iter: Box<dyn Iterator<Item = ProductStoreFields>> =
      Box::new(all_products.into_iter());

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

  pub async fn ticket_types(&self, ctx: &Context<'_>) -> Result<Vec<TicketTypeStoreFields>, Error> {
    let loader_result = load_one_by_model_id!(convention_ticket_types, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, TicketTypeStoreFields))
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
