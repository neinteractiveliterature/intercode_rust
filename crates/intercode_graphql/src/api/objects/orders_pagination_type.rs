use async_graphql::{Context, Error, Object};
use intercode_entities::orders;
use sea_orm::{ConnectionTrait, EntityTrait, Paginator, PaginatorTrait, Select, SelectModel};

use intercode_graphql_core::{query_data::QueryData, ModelBackedType};

use crate::api::interfaces::PaginationImplementation;

use super::OrderType;

pub struct OrdersPaginationType {
  scope: Select<orders::Entity>,
  page: u64,
  per_page: u64,
}

#[Object(name = "OrdersPagination")]
impl OrdersPaginationType {
  #[graphql(name = "current_page")]
  pub async fn current_page(&self) -> u64 {
    self.page
  }

  async fn entries(&self, ctx: &Context<'_>) -> Result<Vec<OrderType>, Error> {
    let db = ctx.data::<QueryData>()?.db();
    let (paginator, _) = self.paginator_and_page_size(db);
    Ok(
      paginator
        .fetch_page(self.page - 1) // sqlx uses 0-based pagination, intercode uses 1-based
        .await?
        .into_iter()
        .map(OrderType::new)
        .collect(),
    )
  }

  #[graphql(name = "per_page")]
  pub async fn per_page(&self) -> u64 {
    self.per_page
  }

  #[graphql(name = "total_entries")]
  pub async fn total_entries(&self, ctx: &Context<'_>) -> Result<u64, Error> {
    <Self as PaginationImplementation<orders::Entity>>::total_entries(self, ctx).await
  }

  #[graphql(name = "total_pages")]
  pub async fn total_pages(&self, ctx: &Context<'_>) -> Result<u64, Error> {
    <Self as PaginationImplementation<orders::Entity>>::total_pages(self, ctx).await
  }
}

impl PaginationImplementation<orders::Entity> for OrdersPaginationType {
  type Selector = SelectModel<orders::Model>;

  fn new(scope: Option<Select<orders::Entity>>, page: Option<u64>, per_page: Option<u64>) -> Self {
    OrdersPaginationType {
      scope: scope.unwrap_or_else(orders::Entity::find),
      page: page.unwrap_or(1),
      per_page: per_page.unwrap_or(20),
    }
  }

  fn paginator_and_page_size<'s, C: ConnectionTrait>(
    &'s self,
    db: &'s C,
  ) -> (Paginator<'s, C, SelectModel<orders::Model>>, u64) {
    (
      self.scope.clone().into_model().paginate(db, self.per_page),
      self.per_page,
    )
  }
}
