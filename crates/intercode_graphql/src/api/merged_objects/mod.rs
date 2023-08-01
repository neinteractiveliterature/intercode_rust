mod event_category_type;
mod event_proposal_type;
mod event_type;
mod form_type;
mod mailing_lists_type;
mod order_entry_type;
mod order_type;
mod permission_type;
mod root_site_type;
mod run_type;
mod signup_request_type;
mod signup_type;
mod team_member_type;
mod ticket_type;
mod user_con_profile_type;

pub use event_category_type::*;
pub use event_proposal_type::*;
pub use event_type::*;
pub use form_type::*;
pub use mailing_lists_type::*;
pub use order_entry_type::*;
pub use order_type::*;
pub use permission_type::*;
pub use root_site_type::*;
pub use run_type::*;
pub use signup_request_type::*;
pub use signup_type::*;
pub use team_member_type::*;
pub use ticket_type::*;
pub use user_con_profile_type::*;

#[macro_export]
macro_rules! merged_model_backed_type {
  ($name: ident, $model: path, $graphql_name: expr, $($types: path),+) => {
    #[derive(MergedObject)]
    #[graphql(name = $graphql_name)]
    pub struct $name($($types),+);

    impl ::intercode_graphql_core::ModelBackedType for $name {
      type Model = $model;

      fn from_arc(arc: ::std::sync::Arc<Self::Model>) -> Self {
        Self(
          $(<$types>::from_arc(arc.clone())),*
        )
      }

      fn new(model: Self::Model) -> Self {
        Self::from_arc(::std::sync::Arc::new(model))
      }

      fn get_model(&self) -> &Self::Model {
        self.0.get_model()
      }

      fn clone_model_arc(&self) -> ::std::sync::Arc<Self::Model> {
        self.0.clone_model_arc()
      }
    }
  };
}
