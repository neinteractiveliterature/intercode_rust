use async_graphql::{Context, Error, Object};
use intercode_entities::events;
use sea_orm::{ConnectionTrait, EntityTrait, Paginator, PaginatorTrait, Select, SelectModel};

use crate::{api::interfaces::PaginationImplementation, QueryData};

use super::{EventType, ModelBackedType};

pub struct EventsPaginationType {
  scope: Select<events::Entity>,
  page: u64,
  per_page: u64,
}

impl EventsPaginationType {
  pub fn new(
    scope: Option<Select<events::Entity>>,
    page: Option<u64>,
    per_page: Option<u64>,
  ) -> Self {
    EventsPaginationType {
      scope: scope.unwrap_or_else(intercode_entities::events::Entity::find),
      page: page.unwrap_or(1),
      per_page: per_page.unwrap_or(20),
    }
  }
}

#[Object(name = "EventsPagination")]
impl EventsPaginationType {
  #[graphql(name = "current_page")]
  async fn current_page(&self) -> u64 {
    self.page
  }

  async fn entries(&self, ctx: &Context<'_>) -> Result<Vec<EventType>, Error> {
    let db = ctx.data::<QueryData>()?.db.as_ref();
    let (paginator, _) = self.paginator_and_page_size(db);
    Ok(
      paginator
        .fetch_page(self.page - 1) // sqlx uses 0-based pagination, intercode uses 1-based
        .await?
        .into_iter()
        .map(EventType::new)
        .collect(),
    )
  }

  #[graphql(name = "per_page")]
  async fn per_page(&self) -> u64 {
    self.per_page
  }

  #[graphql(name = "total_entries")]
  async fn total_entries(&self, ctx: &Context<'_>) -> Result<u64, Error> {
    <Self as PaginationImplementation<SelectModel<events::Model>>>::total_entries(self, ctx).await
  }

  #[graphql(name = "total_pages")]
  async fn total_pages(&self, ctx: &Context<'_>) -> Result<u64, Error> {
    <Self as PaginationImplementation<SelectModel<events::Model>>>::total_pages(self, ctx).await
  }
}

impl PaginationImplementation<SelectModel<events::Model>> for EventsPaginationType {
  fn paginator_and_page_size<'s, C: ConnectionTrait>(
    &'s self,
    db: &'s C,
  ) -> (Paginator<'s, C, SelectModel<events::Model>>, u64) {
    (
      self.scope.clone().into_model().paginate(db, self.per_page),
      self.per_page,
    )
  }
}
