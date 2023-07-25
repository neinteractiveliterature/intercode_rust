use async_graphql::{Context, Error};
use intercode_graphql_core::{ModelBackedType, ModelPaginator, PaginationImplementation};
use intercode_policies::{AuthorizationInfo, EntityPolicy, ReadManageAction};
use intercode_query_builders::QueryBuilder;
use sea_orm::{FromQueryResult, ModelTrait, Select};

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

  fn authorized_from_query_builder<
    B: QueryBuilder<Entity = Model::Entity>,
    P: EntityPolicy<AuthorizationInfo, Model, Action = A>,
    A: From<ReadManageAction>,
  >(
    query_builder: &B,
    ctx: &Context<'_>,
    scope: Select<Model::Entity>,
    page: Option<u64>,
    per_page: Option<u64>,
    _policy: P,
  ) -> Result<Self, Error>
  where
    Self: Sized,
  {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let scope = P::filter_scope(scope, authorization_info, &A::from(ReadManageAction::Read));
    Ok(Self::from_query_builder(
      query_builder,
      scope,
      page,
      per_page,
    ))
  }
}

impl<Item: ModelBackedType> PaginationFromQueryBuilder<Item::Model> for ModelPaginator<Item> where
  Item::Model: Sync + FromQueryResult
{
}
