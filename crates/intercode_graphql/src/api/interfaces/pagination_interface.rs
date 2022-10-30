use async_graphql::{async_trait::async_trait, Context, Error, Interface};
use sea_orm::{DatabaseConnection, Paginator, SelectorTrait};

use crate::{api::objects::EventsPaginationType, SchemaData};

#[derive(Interface)]
#[graphql(
  field(
    name = "total_entries",
    method = "total_entries",
    type = "usize",
    desc = "The total number of items in the paginated list (across all pages)"
  ),
  field(
    name = "total_pages",
    method = "total_pages",
    type = "usize",
    desc = "The total number of pages in the paginated list"
  ),
  field(
    name = "current_page",
    method = "current_page",
    type = "usize",
    desc = "The number of the page currently being returned in this query"
  ),
  field(
    name = "per_page",
    method = "per_page",
    type = "usize",
    desc = "The number of items per page currently being returned in this query"
  )
)]
/// PaginationInterface provides a way to use offset-based pagination on a list of objects. This
/// is useful for UIs such as Intercode's table views, which provide a way to jump between numbered
/// pages.
///
/// Page numbers in PaginationInterface are 1-based (so, the first page is page 1, then page 2,
/// etc.) The number of items per page can be controlled via the per_page argument on paginated
/// fields. It defaults to 20, and can go up to 200.
///
/// Offset-based pagination is different from
/// [Relay's cursor-based pagination](https://relay.dev/graphql/connections.htm) that is more
/// commonly used in GraphQL APIs. We chose to go with an offset-based approach due to our UI
/// needs, but if a cursor-based approach is desirable in the future, we may also implement Relay
/// connections alongside our existing pagination fields.
pub enum PaginationInterface {
  Events(EventsPaginationType),
}

#[async_trait]
pub trait PaginationImplementation<Item: SelectorTrait + Send + Sync> {
  fn paginator_and_page_size<'s>(
    &'s self,
    db: &'s DatabaseConnection,
  ) -> (Paginator<'s, DatabaseConnection, Item>, usize);

  async fn total_entries(&self, ctx: &Context) -> Result<usize, Error> {
    let db = ctx.data::<SchemaData>()?.db.as_ref();
    Ok(self.paginator_and_page_size(db).0.num_items().await?)
  }

  async fn total_pages(&self, ctx: &Context) -> Result<usize, Error> {
    let db = ctx.data::<SchemaData>()?.db.as_ref();
    Ok(self.paginator_and_page_size(db).0.num_pages().await?)
  }

  async fn current_page(&self, ctx: &Context) -> Result<usize, Error> {
    let db = ctx.data::<SchemaData>()?.db.as_ref();
    Ok(self.paginator_and_page_size(db).0.cur_page())
  }

  async fn per_page(&self, ctx: &Context) -> Result<usize, Error> {
    let db = ctx.data::<SchemaData>()?.db.as_ref();
    Ok(self.paginator_and_page_size(db).1)
  }
}