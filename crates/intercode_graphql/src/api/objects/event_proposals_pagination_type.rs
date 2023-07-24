use async_graphql::{Context, Error, Object};
use intercode_entities::event_proposals;
use intercode_graphql_core::{query_data::QueryData, ModelBackedType, PaginationImplementation};
use intercode_pagination_from_query_builder::PaginationFromQueryBuilder;
use sea_orm::{ConnectionTrait, EntityTrait, Paginator, PaginatorTrait, Select, SelectModel};

use crate::api::merged_objects::EventProposalType;

pub struct EventProposalsPaginationType {
  scope: Select<event_proposals::Entity>,
  page: u64,
  per_page: u64,
}

#[Object(name = "EventProposalsPagination")]
impl EventProposalsPaginationType {
  #[graphql(name = "current_page")]
  pub async fn current_page(&self) -> u64 {
    self.page
  }

  async fn entries(&self, ctx: &Context<'_>) -> Result<Vec<EventProposalType>, Error> {
    let db = ctx.data::<QueryData>()?.db();
    let (paginator, _) = self.paginator_and_page_size(db);
    Ok(
      paginator
        .fetch_page(self.page - 1) // sqlx uses 0-based pagination, intercode uses 1-based
        .await?
        .into_iter()
        .map(EventProposalType::new)
        .collect(),
    )
  }

  #[graphql(name = "per_page")]
  pub async fn per_page(&self) -> u64 {
    self.per_page
  }

  #[graphql(name = "total_entries")]
  pub async fn total_entries(&self, ctx: &Context<'_>) -> Result<u64, Error> {
    <Self as PaginationImplementation<event_proposals::Entity>>::total_entries(self, ctx).await
  }

  #[graphql(name = "total_pages")]
  pub async fn total_pages(&self, ctx: &Context<'_>) -> Result<u64, Error> {
    <Self as PaginationImplementation<event_proposals::Entity>>::total_pages(self, ctx).await
  }
}

impl PaginationImplementation<event_proposals::Entity> for EventProposalsPaginationType {
  type Selector = SelectModel<event_proposals::Model>;

  fn new(
    scope: Option<Select<event_proposals::Entity>>,
    page: Option<u64>,
    per_page: Option<u64>,
  ) -> Self {
    EventProposalsPaginationType {
      scope: scope.unwrap_or_else(intercode_entities::event_proposals::Entity::find),
      page: page.unwrap_or(1),
      per_page: per_page.unwrap_or(20),
    }
  }

  fn paginator_and_page_size<'s, C: ConnectionTrait>(
    &'s self,
    db: &'s C,
  ) -> (Paginator<'s, C, SelectModel<event_proposals::Model>>, u64) {
    (
      self.scope.clone().into_model().paginate(db, self.per_page),
      self.per_page,
    )
  }
}

impl PaginationFromQueryBuilder<event_proposals::Entity> for EventProposalsPaginationType {}
