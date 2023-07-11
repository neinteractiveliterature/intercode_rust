use async_graphql::InputObject;
use intercode_graphql_core::scalars::JsonScalar;

#[derive(InputObject, Default)]
pub struct EventFiltersInput {
  pub category: Option<Vec<Option<i64>>>,
  pub title: Option<String>,
  #[graphql(name = "title_prefix")]
  pub title_prefix: Option<String>,
  #[graphql(name = "my_rating")]
  pub my_rating: Option<Vec<i64>>,
  #[graphql(name = "form_items")]
  pub form_items: Option<JsonScalar>,
}
