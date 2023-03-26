use async_graphql::{Context, Error, Object};
use intercode_entities::user_con_profiles;
use sea_orm::{ConnectionTrait, EntityTrait, Paginator, PaginatorTrait, Select, SelectModel};

use crate::{api::interfaces::PaginationImplementation, QueryData};

use super::{ModelBackedType, UserConProfileType};

pub struct UserConProfilesPaginationType {
  scope: Select<user_con_profiles::Entity>,
  page: u64,
  per_page: u64,
}

impl UserConProfilesPaginationType {
  pub fn new(
    scope: Option<Select<user_con_profiles::Entity>>,
    page: Option<u64>,
    per_page: Option<u64>,
  ) -> Self {
    UserConProfilesPaginationType {
      scope: scope.unwrap_or_else(intercode_entities::user_con_profiles::Entity::find),
      page: page.unwrap_or(1),
      per_page: per_page.unwrap_or(20),
    }
  }
}

#[Object(name = "UserConProfilesPagination")]
impl UserConProfilesPaginationType {
  #[graphql(name = "current_page")]
  async fn current_page(&self) -> u64 {
    self.page
  }

  async fn entries(&self, ctx: &Context<'_>) -> Result<Vec<UserConProfileType>, Error> {
    let db = ctx.data::<QueryData>()?.db();
    let (paginator, _) = self.paginator_and_page_size(db);
    Ok(
      paginator
        .fetch_page(self.page - 1) // sqlx uses 0-based pagination, intercode uses 1-based
        .await?
        .into_iter()
        .map(UserConProfileType::new)
        .collect(),
    )
  }

  #[graphql(name = "per_page")]
  async fn per_page(&self) -> u64 {
    self.per_page
  }

  #[graphql(name = "total_entries")]
  async fn total_entries(&self, ctx: &Context<'_>) -> Result<u64, Error> {
    <Self as PaginationImplementation<SelectModel<user_con_profiles::Model>>>::total_entries(
      self, ctx,
    )
    .await
  }

  #[graphql(name = "total_pages")]
  async fn total_pages(&self, ctx: &Context<'_>) -> Result<u64, Error> {
    <Self as PaginationImplementation<SelectModel<user_con_profiles::Model>>>::total_pages(
      self, ctx,
    )
    .await
  }
}

impl PaginationImplementation<SelectModel<user_con_profiles::Model>>
  for UserConProfilesPaginationType
{
  fn paginator_and_page_size<'s, C: ConnectionTrait>(
    &'s self,
    db: &'s C,
  ) -> (Paginator<'s, C, SelectModel<user_con_profiles::Model>>, u64) {
    (
      self.scope.clone().into_model().paginate(db, self.per_page),
      self.per_page,
    )
  }
}
