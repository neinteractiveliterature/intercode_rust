use async_graphql::*;

pub struct SearchResultType;

#[Object(name = "SearchResult")]
impl SearchResultType {
  async fn total_entries(&self) -> usize {
    0
  }
}
