mod email_routes_query_builder;
pub mod sort_input;

pub use email_routes_query_builder::*;
use intercode_graphql_core::{ModelBackedType, ModelPaginator, PaginationImplementation};

use sea_orm::{EntityTrait, FromQueryResult, ModelTrait, Select};

pub trait QueryBuilder {
  type Entity: EntityTrait;

  fn apply_filters(&self, scope: Select<Self::Entity>) -> Select<Self::Entity>;
  fn apply_sorts(&self, scope: Select<Self::Entity>) -> Select<Self::Entity>;
}

pub trait PaginationFromQueryBuilder<Model: ModelTrait>: PaginationImplementation<Model>
where
  Model: Sync,
{
  fn from_query_builder<B: QueryBuilder<Entity = Model::Entity>>(
    query_builder: &B,
    scope: Select<Model::Entity>,
    page: Option<u64>,
    per_page: Option<u64>,
  ) -> Self
  where
    Self: Sized,
  {
    let scope = query_builder.apply_filters(scope);
    let scope = query_builder.apply_sorts(scope);

    Self::new(Some(scope), page, per_page)
  }
}

impl<Item: ModelBackedType> PaginationFromQueryBuilder<Item::Model> for ModelPaginator<Item> where
  Item::Model: Sync + FromQueryResult
{
}
