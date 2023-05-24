use async_graphql::InputObject;

#[derive(InputObject, Default)]
pub struct EventProposalFiltersInput {
  #[graphql(name = "event_category")]
  pub event_category: Option<Vec<Option<i64>>>,
  pub title: Option<String>,
  pub owner: Option<String>,
  pub status: Option<Vec<Option<String>>>,
}
