mod email_routes_query_builder;
mod event_proposals_query_builder;
mod events_query_builder;
mod signup_requests_query_builder;
pub mod sort_input;
mod user_con_profiles_query_builder;

pub use email_routes_query_builder::*;
pub use event_proposals_query_builder::*;
pub use events_query_builder::*;
pub use signup_requests_query_builder::*;
pub use user_con_profiles_query_builder::*;

use sea_orm::{EntityTrait, Select};

pub trait QueryBuilder {
  type Entity: EntityTrait;

  fn apply_filters(&self, scope: Select<Self::Entity>) -> Select<Self::Entity>;
  fn apply_sorts(&self, scope: Select<Self::Entity>) -> Select<Self::Entity>;
}
