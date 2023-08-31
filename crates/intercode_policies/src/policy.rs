use std::fmt::Display;

use axum::async_trait;
use intercode_graphql_core::ModelBackedType;
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter, QuerySelect, Select};

use crate::{AuthorizationInfo, PolicyGuard};

#[derive(PartialEq, Eq)]
pub enum CRUDAction {
  Create,
  Read,
  Update,
  Delete,
}

#[derive(PartialEq, Eq)]
pub enum ReadManageAction {
  Read,
  Manage,
}

impl From<CRUDAction> for ReadManageAction {
  fn from(action: CRUDAction) -> Self {
    match action {
      CRUDAction::Read => Self::Read,
      CRUDAction::Create | CRUDAction::Update | CRUDAction::Delete => Self::Manage,
    }
  }
}

#[async_trait]
pub trait Policy<Principal: Send + Sync, Resource: Send + Sync> {
  type Action: Send + Sync;
  type Error: Send + Sync + Display;

  async fn action_permitted(
    principal: &Principal,
    action: &Self::Action,
    resource: &Resource,
  ) -> Result<bool, Self::Error>;

  async fn permitted<A: Into<Self::Action> + Send>(
    principal: &Principal,
    action: A,
    resource: &Resource,
  ) -> Result<bool, Self::Error> {
    Self::action_permitted(principal, &action.into(), resource).await
  }
}

pub trait EntityPolicy<Principal: Send + Sync, Resource: ModelTrait + Sync> {
  type Action: Send + Sync;
  fn accessible_to(principal: &Principal, action: &Self::Action) -> Select<Resource::Entity>;
  fn id_column() -> <Resource::Entity as EntityTrait>::Column;

  fn filter_scope(
    scope: Select<Resource::Entity>,
    principal: &Principal,
    action: &Self::Action,
  ) -> Select<Resource::Entity> {
    let id_column = Self::id_column();

    scope.filter(
      id_column.in_subquery(
        sea_orm::QuerySelect::query(
          &mut Self::accessible_to(principal, action)
            .select_only()
            .column(id_column),
        )
        .take(),
      ),
    )
  }
}

pub trait GuardablePolicy<'a, Resource: Send + Sync + Clone, Model: Send + Sync + Clone + 'static>:
  Policy<AuthorizationInfo, Resource> + Sized
{
  type Guard: PolicyGuard<'a, Self, Resource, Model> + Send + Sync + 'static;

  fn guard(
    action: Self::Action,
    model: &'a Model,
  ) -> Box<dyn PolicyGuard<'a, Self, Resource, Model> + Send + Sync> {
    Box::new(Self::Guard::new(action, model))
  }
}

pub trait ModelBackedTypeGuardablePolicy<'a, Resource: Send + Sync + Clone, T: ModelBackedType>:
  GuardablePolicy<'a, Resource, T::Model>
where
  T::Model: Send + Sync + 'static,
{
  fn model_guard(
    action: Self::Action,
    mbt: &'a T,
  ) -> Box<dyn PolicyGuard<'a, Self, Resource, T::Model> + Send + Sync> {
    Box::new(Self::Guard::new(action, mbt.get_model()))
  }
}

impl<
    'a,
    Resource: Send + Sync + Clone,
    T: ModelBackedType,
    P: GuardablePolicy<'a, Resource, T::Model>,
  > ModelBackedTypeGuardablePolicy<'a, Resource, T> for P
where
  T::Model: Sync + 'static,
{
}
