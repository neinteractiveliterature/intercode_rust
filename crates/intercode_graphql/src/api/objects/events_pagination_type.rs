use async_graphql::{Context, Error, Object};
use intercode_entities::events;
use sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait, Select, SelectModel};

use crate::{api::interfaces::PaginationImplementation, SchemaData};

use super::{EventType, ModelBackedType};

pub struct EventsPaginationType {
  scope: Select<events::Entity>,
  page: usize,
  per_page: usize,
}

impl EventsPaginationType {
  pub fn new(
    scope: Option<Select<events::Entity>>,
    page: Option<usize>,
    per_page: Option<usize>,
  ) -> Self {
    EventsPaginationType {
      scope: scope.unwrap_or_else(intercode_entities::events::Entity::find),
      page: page.unwrap_or(1),
      per_page: per_page.unwrap_or(20),
    }
  }
}

#[Object]
impl EventsPaginationType {
  async fn entries(&self, ctx: &Context<'_>) -> Result<Vec<EventType>, Error> {
    let db = ctx.data::<SchemaData>()?.db.as_ref();
    let (paginator, _) = self.paginator_and_page_size(db);
    Ok(
      paginator
        .fetch_page(self.page)
        .await?
        .into_iter()
        .map(EventType::new)
        .collect(),
    )
  }
}

impl PaginationImplementation<SelectModel<events::Model>> for EventsPaginationType {
  fn paginator_and_page_size<'s>(
    &'s self,
    db: &'s DatabaseConnection,
  ) -> (
    sea_orm::Paginator<'s, DatabaseConnection, SelectModel<events::Model>>,
    usize,
  ) {
    (
      self.scope.clone().into_model().paginate(db, self.per_page),
      self.per_page,
    )
  }
}
