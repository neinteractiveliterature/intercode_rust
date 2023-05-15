use async_graphql::InputObject;

#[derive(InputObject, Default)]
pub struct CouponFiltersInput {
  pub code: Option<String>,
}
