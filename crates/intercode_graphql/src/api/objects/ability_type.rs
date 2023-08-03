use std::sync::Arc;

use async_graphql::*;
use intercode_cms::api::partial_objects::AbilityCmsFields;
use intercode_conventions::partial_objects::AbilityConventionsFields;
use intercode_entities::email_routes;
use intercode_events::partial_objects::AbilityEventsFields;
use intercode_forms::partial_objects::AbilityFormsFields;

use intercode_policies::{policies::EmailRoutePolicy, AuthorizationInfo, Policy, ReadManageAction};
use intercode_reporting::partial_objects::AbilityReportingFields;
use intercode_signups::partial_objects::AbilitySignupsFields;
use intercode_store::partial_objects::AbilityStoreFields;
use intercode_users::partial_objects::AbilityUsersFields;

pub struct AbilityApiFields {
  authorization_info: Arc<AuthorizationInfo>,
}

impl AbilityApiFields {
  pub fn new(authorization_info: Arc<AuthorizationInfo>) -> Self {
    Self { authorization_info }
  }
}

#[Object]
impl AbilityApiFields {
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

  #[graphql(name = "can_manage_oauth_applications")]
  async fn can_manage_oauth_applications(&self) -> bool {
    // TODO
    false
  }
}

#[derive(MergedObject)]
#[graphql(name = "Ability")]
pub struct AbilityType(
  AbilityConventionsFields,
  AbilityEventsFields,
  AbilityStoreFields,
  AbilityCmsFields,
  AbilityFormsFields,
  AbilityReportingFields,
  AbilitySignupsFields,
  AbilityUsersFields,
  AbilityApiFields,
);

impl AbilityType {
  pub fn new(authorization_info: Arc<AuthorizationInfo>) -> Self {
    Self(
      AbilityConventionsFields::new(authorization_info.clone()),
      AbilityEventsFields::new(authorization_info.clone()),
      AbilityStoreFields::new(authorization_info.clone()),
      AbilityCmsFields::new(authorization_info.clone()),
      AbilityFormsFields::new(authorization_info.clone()),
      AbilityReportingFields::new(authorization_info.clone()),
      AbilitySignupsFields::new(authorization_info.clone()),
      AbilityUsersFields::new(authorization_info.clone()),
      AbilityApiFields::new(authorization_info),
    )
  }
}

impl From<AbilityUsersFields> for AbilityType {
  fn from(value: AbilityUsersFields) -> Self {
    Self::new(value.into_authorization_info())
  }
}
