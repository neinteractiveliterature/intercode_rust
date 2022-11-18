use std::fmt::Display;

use axum::async_trait;
use sea_orm::{ModelTrait, Select};

pub enum CRUDAction {
  Create,
  Read,
  Update,
  Delete,
}

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

pub trait EntityPolicy<Principal: Send + Sync, Resource: ModelTrait + Sync>:
  Policy<Principal, Resource>
{
  fn accessible_to(principal: &Principal) -> Select<Resource::Entity>;
}
