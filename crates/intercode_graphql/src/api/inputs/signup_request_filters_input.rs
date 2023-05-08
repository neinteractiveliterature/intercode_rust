use async_graphql::InputObject;

#[derive(InputObject, Default)]
pub struct SignupRequestFiltersInput {
  pub state: Option<Vec<String>>,
}
