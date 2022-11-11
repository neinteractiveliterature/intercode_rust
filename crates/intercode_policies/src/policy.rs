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
pub trait Policy<Principal: Send + Sync, Action: Send, Resource: Send + Sync> {
  type Error;

  async fn action_permitted(
    principal: &Principal,
    action: Action,
    resource: &Resource,
  ) -> Result<bool, Self::Error>;

  async fn permitted<A: Into<Action> + Send>(
    principal: &Principal,
    action: A,
    resource: &Resource,
  ) -> Result<bool, Self::Error> {
    Self::action_permitted(principal, action.into(), resource).await
  }
}

pub trait EntityPolicy<Principal: Send + Sync, Action: Send, Resource: ModelTrait + Sync>:
  Policy<Principal, Action, Resource>
{
  fn accessible_to(principal: &Principal) -> Select<Resource::Entity>;
}
