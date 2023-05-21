use async_graphql::InputObject;

#[derive(InputObject, Default)]
pub struct EmailRouteFiltersInput {
  #[graphql(name = "receiver_address")]
  pub receiver_address: Option<String>,
  #[graphql(name = "forward_addresses")]
  pub forward_addresses: Option<String>,
}
