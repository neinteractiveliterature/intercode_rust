use async_graphql::Enum;
use strum::EnumString;

#[derive(Enum, Copy, Clone, Eq, PartialEq, EnumString, Default)]
#[strum(serialize_all = "snake_case")]
pub enum SignupChangeAction {
  #[graphql(name = "accept_signup_request")]
  AcceptSignupRequest,
  #[graphql(name = "admin_create_signup")]
  AdminCreateSignup,
  #[graphql(name = "change_registration_policy")]
  ChangeRegistrationPolicy,
  #[graphql(name = "hold_expired")]
  HoldExpired,
  #[graphql(name = "self_service_signup")]
  SelfServiceSignup,
  #[graphql(name = "ticket_purchase")]
  TicketPurchase,
  #[default]
  #[graphql(name = "unknown")]
  Unknown,
  #[graphql(name = "vacancy_fill")]
  VacancyFill,
  #[graphql(name = "withdraw")]
  Withdraw,
}
