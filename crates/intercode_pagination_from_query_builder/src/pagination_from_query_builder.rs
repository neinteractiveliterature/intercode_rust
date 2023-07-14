use async_graphql::{Context, Error};
use intercode_graphql_core::PaginationImplementation;
use intercode_policies::{AuthorizationInfo, EntityPolicy, ReadManageAction};
use intercode_query_builders::QueryBuilder;
use sea_orm::{EntityTrait, Select};

pub trait PaginationFromQueryBuilder<Entity: EntityTrait>:
  PaginationImplementation<Entity>
{
  fn from_query_builder<B: QueryBuilder<Entity = Entity>>(
    query_builder: &B,
    scope: Select<Entity>,
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
    B: QueryBuilder<Entity = Entity>,
    P: EntityPolicy<AuthorizationInfo, <Entity as EntityTrait>::Model, Action = A>,
    A: From<ReadManageAction>,
  >(
    query_builder: &B,
    ctx: &Context<'_>,
    scope: Select<Entity>,
    page: Option<u64>,
    per_page: Option<u64>,
    _policy: P,
  ) -> Result<Self, Error>
  where
    <Entity as EntityTrait>::Model: Sync,
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
