use std::sync::Arc;

use async_graphql::*;
use intercode_graphql_core::query_data::QueryData;
use intercode_policies::{
  model_action_permitted::model_action_permitted,
  policies::{ConventionAction, ConventionPolicy},
  AuthorizationInfo, Policy,
};

pub struct AbilityReportingFields {
  authorization_info: Arc<AuthorizationInfo>,
}

impl AbilityReportingFields {
  pub fn new(authorization_info: Arc<AuthorizationInfo>) -> Self {
    Self { authorization_info }
  }
}

#[Object]
impl AbilityReportingFields {
  #[graphql(name = "can_read_any_mailing_list")]
  async fn can_read_any_mailing_list(&self, ctx: &Context<'_>) -> Result<bool> {
    let Some(convention) = ctx.data::<QueryData>()?.convention() else {
      return Ok(false);
    };

    Ok(
      ConventionPolicy::action_permitted(
        &self.authorization_info,
        &ConventionAction::ReadAnyMailingList,
        convention,
      )
      .await?,
    )
  }

  #[graphql(name = "can_read_reports")]
  async fn can_read_reports(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    model_action_permitted(
      self.authorization_info.as_ref(),
      ConventionPolicy,
      ctx,
      &ConventionAction::ViewReports,
      |ctx| Ok(ctx.data::<QueryData>()?.convention()),
    )
    .await
  }
}
