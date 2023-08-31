use std::sync::Arc;

use async_graphql::*;
use intercode_entities::email_routes;
use intercode_policies::{AuthorizationInfo, Policy, ReadManageAction};

use crate::policies::EmailRoutePolicy;

pub struct AbilityEmailFields {
  authorization_info: Arc<AuthorizationInfo>,
}

impl AbilityEmailFields {
  pub fn new(authorization_info: Arc<AuthorizationInfo>) -> Self {
    Self { authorization_info }
  }
}

#[Object]
impl AbilityEmailFields {
  #[graphql(name = "can_manage_email_routes")]
  async fn can_manage_email_routes(&self) -> Result<bool> {
    Ok(
      EmailRoutePolicy::action_permitted(
        self.authorization_info.as_ref(),
        &ReadManageAction::Manage,
        &email_routes::Model::default(),
      )
      .await?,
    )
  }
}
