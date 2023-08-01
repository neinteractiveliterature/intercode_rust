use crate::api::merged_objects::{OrderType, SignupType, TeamMemberType, TicketType};
use crate::api::objects::{AbilityType, ConventionType, StaffPositionType};
use crate::merged_model_backed_type;
use async_graphql::*;
use intercode_entities::user_con_profiles;
use intercode_forms::partial_objects::UserConProfileFormsFields;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_policies::policies::{UserConProfileAction, UserConProfilePolicy};
use intercode_policies::ModelBackedTypeGuardablePolicy;
use intercode_store::partial_objects::UserConProfileStoreFields;
use intercode_users::partial_objects::UserConProfileUsersFields;

model_backed_type!(UserConProfileGlueFields, user_con_profiles::Model);

#[Object(guard = "UserConProfilePolicy::model_guard(UserConProfileAction::Read, self)")]
impl UserConProfileGlueFields {
  async fn ability(&self, ctx: &Context<'_>) -> Result<AbilityType> {
    UserConProfileUsersFields::from_type(self.clone())
      .ability(ctx)
      .await
      .map(AbilityType::from)
  }

  async fn convention(&self, ctx: &Context<'_>) -> Result<ConventionType, Error> {
    UserConProfileUsersFields::from_type(self.clone())
      .convention(ctx)
      .await
      .map(ConventionType::new)
  }

  #[graphql(name = "current_pending_order")]
  async fn current_pending_order(&self, ctx: &Context<'_>) -> Result<Option<OrderType>, Error> {
    UserConProfileStoreFields::from_type(self.clone())
      .current_pending_order(ctx)
      .await
      .map(|res| res.map(OrderType::from_type))
  }

  async fn signups(&self, ctx: &Context<'_>) -> Result<Vec<SignupType>> {
    UserConProfileUsersFields::from_type(self.clone())
      .signups(ctx)
      .await
      .map(|res| res.into_iter().map(SignupType::new).collect())
  }

  #[graphql(name = "staff_positions")]
  async fn staff_positions(&self, ctx: &Context<'_>) -> Result<Vec<StaffPositionType>, Error> {
    UserConProfileUsersFields::from_type(self.clone())
      .staff_positions(ctx)
      .await
      .map(|res| res.into_iter().map(StaffPositionType::new).collect())
  }

  #[graphql(name = "team_members")]
  async fn team_members(&self, ctx: &Context<'_>) -> Result<Vec<TeamMemberType>, Error> {
    UserConProfileUsersFields::from_type(self.clone())
      .team_members(ctx)
      .await
      .map(|res| res.into_iter().map(TeamMemberType::new).collect())
  }

  async fn ticket(&self, ctx: &Context<'_>) -> Result<Option<TicketType>, Error> {
    UserConProfileUsersFields::from_type(self.clone())
      .ticket(ctx)
      .await
      .map(|res| res.map(TicketType::new))
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
