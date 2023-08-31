use async_graphql::*;
use intercode_entities::forms;
use intercode_graphql_core::query_data::QueryData;
use intercode_policies::{AuthorizationInfo, Policy, ReadManageAction};
use std::sync::Arc;

use crate::policies::FormPolicy;

pub struct AbilityFormsFields {
  authorization_info: Arc<AuthorizationInfo>,
}

impl AbilityFormsFields {
  pub fn new(authorization_info: Arc<AuthorizationInfo>) -> Self {
    Self { authorization_info }
  }
}

#[Object]
impl AbilityFormsFields {
  #[graphql(name = "can_manage_forms")]
  async fn can_manage_forms(&self, ctx: &Context<'_>) -> Result<bool> {
    let convention = ctx.data::<QueryData>()?.convention();

    Ok(
      FormPolicy::action_permitted(
        self.authorization_info.as_ref(),
        &ReadManageAction::Manage,
        &forms::Model {
          convention_id: convention.map(|convention| convention.id),
          ..Default::default()
        },
      )
      .await?,
    )
  }
}
