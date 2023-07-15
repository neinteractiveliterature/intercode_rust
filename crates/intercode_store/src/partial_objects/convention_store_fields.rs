use async_graphql::*;
use intercode_entities::{conventions, coupons, links::ConventionToOrders};
use intercode_graphql_core::{load_one_by_model_id, loader_result_to_many, model_backed_type};
use intercode_pagination_from_query_builder::PaginationFromQueryBuilder;
use intercode_policies::policies::{CouponPolicy, OrderPolicy};
use intercode_query_builders::{
  sort_input::SortInput, CouponFiltersInput, CouponsQueryBuilder, OrderFiltersInput,
  OrdersQueryBuilder,
};
use sea_orm::ModelTrait;

use crate::objects::{CouponsPaginationType, OrdersPaginationType, ProductType, TicketTypeType};

model_backed_type!(ConventionStoreFields, conventions::Model);

#[Object]
impl ConventionStoreFields {
  #[graphql(name = "coupons_paginated")]
  async fn coupons_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    #[graphql(name = "per_page")] per_page: Option<u64>,
    filters: Option<CouponFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<CouponsPaginationType, Error> {
    CouponsPaginationType::authorized_from_query_builder(
      &CouponsQueryBuilder::new(filters, sort),
      ctx,
      self.model.find_related(coupons::Entity),
      page,
      per_page,
      CouponPolicy,
    )
  }

  #[graphql(name = "orders_paginated")]
  async fn orders_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    #[graphql(name = "per_page")] per_page: Option<u64>,
    filters: Option<OrderFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<OrdersPaginationType, Error> {
    OrdersPaginationType::authorized_from_query_builder(
      &OrdersQueryBuilder::new(filters, sort),
      ctx,
      self.model.find_linked(ConventionToOrders),
      page,
      per_page,
      OrderPolicy,
    )
  }

  async fn products(&self, ctx: &Context<'_>) -> Result<Vec<ProductType>> {
    let loader_result = load_one_by_model_id!(convention_products, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, ProductType))
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
