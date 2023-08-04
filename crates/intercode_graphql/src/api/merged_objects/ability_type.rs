use std::sync::Arc;

use async_graphql::*;
use intercode_cms::api::partial_objects::AbilityCmsFields;
use intercode_conventions::partial_objects::AbilityConventionsFields;
use intercode_email::partial_objects::AbilityEmailFields;
use intercode_events::partial_objects::AbilityEventsFields;
use intercode_forms::partial_objects::AbilityFormsFields;

use intercode_policies::AuthorizationInfo;
use intercode_reporting::partial_objects::AbilityReportingFields;
use intercode_signups::partial_objects::AbilitySignupsFields;
use intercode_store::partial_objects::AbilityStoreFields;
use intercode_users::partial_objects::AbilityUsersFields;

#[derive(MergedObject)]
#[graphql(name = "Ability")]
pub struct AbilityType(
  AbilityConventionsFields,
  AbilityEmailFields,
  AbilityEventsFields,
  AbilityStoreFields,
  AbilityCmsFields,
  AbilityFormsFields,
  AbilityReportingFields,
  AbilitySignupsFields,
  AbilityUsersFields,
);

impl AbilityType {
  pub fn new(authorization_info: Arc<AuthorizationInfo>) -> Self {
    Self(
      AbilityConventionsFields::new(authorization_info.clone()),
      AbilityEmailFields::new(authorization_info.clone()),
      AbilityEventsFields::new(authorization_info.clone()),
      AbilityStoreFields::new(authorization_info.clone()),
      AbilityCmsFields::new(authorization_info.clone()),
      AbilityFormsFields::new(authorization_info.clone()),
      AbilityReportingFields::new(authorization_info.clone()),
      AbilitySignupsFields::new(authorization_info.clone()),
      AbilityUsersFields::new(authorization_info.clone()),
    )
  }
}

impl From<AbilityUsersFields> for AbilityType {
  fn from(value: AbilityUsersFields) -> Self {
    Self::new(value.into_authorization_info())
  }
}
