use async_graphql::{Context, Error, InputObject};
use intercode_entities::{events, runs, signups, user_con_profiles, users};
use sea_orm::{ColumnTrait, Condition, JoinType, QueryFilter, QuerySelect, RelationTrait, Select};

use crate::filter_utils::string_search_condition;

#[derive(InputObject, Default)]
pub struct SignupFiltersInput {
  name: Option<String>,
  #[graphql(name = "event_title")]
  event_title: Option<String>,
  bucket: Option<Vec<String>>,
  email: Option<String>,
  state: Option<Vec<String>>,
}

impl SignupFiltersInput {
  pub fn apply_filters(
    &self,
    _ctx: &Context<'_>,
    scope: &Select<signups::Entity>,
  ) -> Result<Select<signups::Entity>, Error> {
    let mut scope = scope.clone();

    if let Some(name) = &self.name {
      scope = scope
        .join(
          JoinType::InnerJoin,
          signups::Relation::UserConProfiles.def(),
        )
        .filter(
          Condition::any()
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

    if let Some(event_title) = &self.event_title {
      scope = scope
        .join(JoinType::InnerJoin, signups::Relation::Runs.def())
        .join(JoinType::InnerJoin, runs::Relation::Events.def())
        .filter(string_search_condition(event_title, events::Column::Title));
    }

    if let Some(bucket) = &self.bucket {
      scope = scope.filter(signups::Column::BucketKey.is_in(bucket.iter().map(|b| b.as_str())));
    }

    if let Some(email) = &self.email {
      scope = scope
        .join(
          JoinType::InnerJoin,
          signups::Relation::UserConProfiles.def(),
        )
        .join(JoinType::InnerJoin, signups::Relation::Users.def())
        .filter(string_search_condition(email, users::Column::Email))
    }

    if let Some(state) = &self.state {
      scope = scope.filter(signups::Column::State.is_in(state.iter().map(|s| s.as_str())))
    }

    Ok(scope)
  }
}
