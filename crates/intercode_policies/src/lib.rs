mod authorization_info;
pub mod model_action_permitted;
mod permissions_loading;
pub mod policies;
mod policy;
mod policy_guard;
mod simple_policy_guard;
#[cfg(feature = "test_helpers")]
pub mod test_helpers;

use async_graphql::{Context, Error};
pub use authorization_info::*;
use intercode_graphql_core::{ModelBackedType, ModelPaginator};
use intercode_query_builders::{PaginationFromQueryBuilder, QueryBuilder};
pub use permissions_loading::*;
pub use policy::*;
pub use policy_guard::*;
use sea_orm::{FromQueryResult, ModelTrait, Select};
pub use simple_policy_guard::*;

pub trait AuthorizedFromQueryBuilder<Model: ModelTrait>: PaginationFromQueryBuilder<Model>
where
  Model: Sync,
{
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

impl<Item: ModelBackedType> AuthorizedFromQueryBuilder<Item::Model> for ModelPaginator<Item> where
  Item::Model: Sync + FromQueryResult
{
}
