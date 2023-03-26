use async_graphql::{Context, Error, InputObject, ID};
use intercode_entities::user_con_profiles;
use sea_orm::{sea_query::Cond, QueryFilter, Select};

use crate::filter_utils::string_search_condition;

#[derive(InputObject, Default)]
pub struct UserConProfileFiltersInput {
  id: Option<ID>,
  attending: Option<bool>,
  email: Option<String>,
  #[graphql(name = "first_name")]
  first_name: Option<String>,
  #[graphql(name = "is_team_member")]
  is_team_member: Option<bool>,
  #[graphql(name = "last_name")]
  last_name: Option<String>,
  #[graphql(name = "payment_amount")]
  payment_amount: Option<f64>,
  privileges: Option<String>,
  name: Option<String>,
  #[graphql(name = "event_title")]
  ticket: Option<Vec<ID>>,
  #[graphql(name = "ticket_type")]
  ticket_type: Option<Vec<ID>>,
  user_id: Option<ID>,
}

impl UserConProfileFiltersInput {
  pub fn apply_filters(
    &self,
    _ctx: &Context<'_>,
    scope: &Select<user_con_profiles::Entity>,
  ) -> Result<Select<user_con_profiles::Entity>, Error> {
    let mut scope = scope.clone();

    // TODO implement the remaining filters

    if let Some(first_name) = &self.first_name {
      scope = scope.filter(string_search_condition(
        first_name,
        user_con_profiles::Column::FirstName,
      ));
    }

    if let Some(last_name) = &self.last_name {
      scope = scope.filter(string_search_condition(
        last_name,
        user_con_profiles::Column::LastName,
      ));
    }

    if let Some(name) = &self.name {
      scope = scope.filter(
        Cond::any()
          .add(string_search_condition(
            name,
            user_con_profiles::Column::FirstName,
          ))
          .add(string_search_condition(
            name,
            user_con_profiles::Column::LastName,
          )),
      );
    }

    Ok(scope)
  }
}
