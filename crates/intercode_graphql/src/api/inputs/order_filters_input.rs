use async_graphql::{InputObject, ID};

#[derive(InputObject, Default)]
pub struct OrderFiltersInput {
  pub id: Option<ID>,
  #[graphql(name = "user_name")]
  pub user_name: Option<String>,
  pub status: Option<Vec<String>>,
}
