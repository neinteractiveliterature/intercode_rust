use async_graphql::{Context, Error};
use async_trait::async_trait;
use sea_orm::{ConnectionTrait, EntityTrait, Paginator, Select, SelectorTrait};

use crate::query_data::QueryData;

#[async_trait]
pub trait PaginationImplementation<Entity: EntityTrait + Send + Sync> {
  type Selector: SelectorTrait<Item = Entity::Model> + Send + Sync;

  fn new(scope: Option<Select<Entity>>, page: Option<u64>, per_page: Option<u64>) -> Self;

  fn paginator_and_page_size<'s, C: ConnectionTrait>(
    &'s self,
    db: &'s C,
  ) -> (Paginator<'s, C, Self::Selector>, u64);

  async fn total_entries(&self, ctx: &Context) -> Result<u64, Error> {
    let db = ctx.data::<QueryData>()?.db();
    Ok(self.paginator_and_page_size(db).0.num_items().await?)
  }

  async fn total_pages(&self, ctx: &Context) -> Result<u64, Error> {
    let db = ctx.data::<QueryData>()?.db();
    Ok(self.paginator_and_page_size(db).0.num_pages().await?)
  }

  async fn current_page(&self, ctx: &Context) -> Result<u64, Error> {
    let db = ctx.data::<QueryData>()?.db();
    Ok(self.paginator_and_page_size(db).0.cur_page())
  }

  async fn per_page(&self, ctx: &Context) -> Result<u64, Error> {
    let db = ctx.data::<QueryData>()?.db();
    Ok(self.paginator_and_page_size(db).1)
  }
}
