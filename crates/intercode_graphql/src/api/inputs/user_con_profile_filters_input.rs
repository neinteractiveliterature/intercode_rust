use async_graphql::{InputObject, ID};

#[derive(InputObject, Default)]
pub struct UserConProfileFiltersInput {
  pub id: Option<ID>,
  pub attending: Option<bool>,
  pub email: Option<String>,
  #[graphql(name = "first_name")]
  pub first_name: Option<String>,
  #[graphql(name = "is_team_member")]
  pub is_team_member: Option<bool>,
  #[graphql(name = "last_name")]
  pub last_name: Option<String>,
  #[graphql(name = "payment_amount")]
  pub payment_amount: Option<f64>,
  pub privileges: Option<String>,
  pub name: Option<String>,
  #[graphql(name = "event_title")]
  pub ticket: Option<Vec<ID>>,
  #[graphql(name = "ticket_type")]
  pub ticket_type: Option<Vec<ID>>,
  pub user_id: Option<ID>,
}
