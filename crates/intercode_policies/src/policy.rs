use std::fmt::Display;

use axum::async_trait;
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter, QuerySelect, Select};

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
