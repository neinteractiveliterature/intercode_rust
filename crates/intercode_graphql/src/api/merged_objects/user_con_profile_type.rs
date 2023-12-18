use crate::api::merged_objects::{OrderType, SignupType, TeamMemberType, TicketType};
use crate::merged_model_backed_type;
use async_graphql::*;
use intercode_entities::user_con_profiles;
use intercode_forms::partial_objects::UserConProfileFormsFields;
use intercode_graphql_core::model_backed_type;
use intercode_policies::policies::{UserConProfileAction, UserConProfilePolicy};
use intercode_policies::ModelBackedTypeGuardablePolicy;
use intercode_store::partial_objects::{UserConProfileStoreExtensions, UserConProfileStoreFields};
use intercode_users::partial_objects::{UserConProfileUsersExtensions, UserConProfileUsersFields};

use super::{AbilityType, ConventionType, SignupRequestType, StaffPositionType, UserType};

model_backed_type!(UserConProfileGlueFields, user_con_profiles::Model);

impl UserConProfileStoreExtensions for UserConProfileGlueFields {}
impl UserConProfileUsersExtensions for UserConProfileGlueFields {}

#[Object(guard = "UserConProfilePolicy::model_guard(UserConProfileAction::Read, self)")]
impl UserConProfileGlueFields {
  async fn ability(&self, ctx: &Context<'_>) -> Result<AbilityType> {
    UserConProfileUsersExtensions::ability(self, ctx)
      .await
      .map(AbilityType::from)
  }

  async fn convention(&self, ctx: &Context<'_>) -> Result<ConventionType, Error> {
    UserConProfileUsersExtensions::convention(self, ctx).await
  }

  #[graphql(name = "current_pending_order")]
  async fn current_pending_order(&self, ctx: &Context<'_>) -> Result<Option<OrderType>, Error> {
    UserConProfileStoreExtensions::current_pending_order(self, ctx).await
  }

  async fn orders(&self, ctx: &Context<'_>) -> Result<Vec<OrderType>, Error> {
    UserConProfileStoreExtensions::orders(self, ctx).await
  }

  async fn signups(&self, ctx: &Context<'_>) -> Result<Vec<SignupType>> {
    UserConProfileUsersExtensions::signups(self, ctx).await
  }

  #[graphql(name = "signup_requests")]
  async fn signup_requests(&self, ctx: &Context<'_>) -> Result<Vec<SignupRequestType>> {
    UserConProfileUsersExtensions::signup_requests(self, ctx).await
  }

  #[graphql(name = "staff_positions")]
  async fn staff_positions(&self, ctx: &Context<'_>) -> Result<Vec<StaffPositionType>, Error> {
    UserConProfileUsersExtensions::staff_positions(self, ctx).await
  }

  #[graphql(name = "team_members")]
  async fn team_members(&self, ctx: &Context<'_>) -> Result<Vec<TeamMemberType>, Error> {
    UserConProfileUsersExtensions::team_members(self, ctx).await
  }

  async fn ticket(&self, ctx: &Context<'_>) -> Result<Option<TicketType>, Error> {
    UserConProfileUsersExtensions::ticket(self, ctx).await
  }

  async fn user(&self, ctx: &Context<'_>) -> Result<UserType, Error> {
    UserConProfileUsersExtensions::user(self, ctx).await
  }
}

merged_model_backed_type!(
  UserConProfileType,
  user_con_profiles::Model,
  "UserConProfile",
  UserConProfileGlueFields,
  UserConProfileUsersFields,
  UserConProfileFormsFields,
  UserConProfileStoreFields
);
