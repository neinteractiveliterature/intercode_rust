use async_graphql::*;
use intercode_entities::forms;
use intercode_graphql_core::query_data::QueryData;
use intercode_policies::{policies::FormPolicy, AuthorizationInfo, Policy, ReadManageAction};
use std::borrow::Cow;

pub struct AbilityFormsFields<'a> {
  authorization_info: Cow<'a, AuthorizationInfo>,
}

impl<'a> AbilityFormsFields<'a> {
  pub fn new(authorization_info: Cow<'a, AuthorizationInfo>) -> Self {
    Self { authorization_info }
  }
}

#[Object]
impl<'a> AbilityFormsFields<'a> {
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
