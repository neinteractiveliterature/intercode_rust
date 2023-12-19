mod ability_type;
mod cms_content_group_type;
mod convention_type;
mod coupon_application_type;
mod coupon_type;
mod department_type;
mod event_category_type;
mod event_proposal_type;
mod event_type;
mod form_item_type;
mod form_response_change_type;
mod form_section_type;
mod form_type;
mod mailing_lists_type;
mod maximum_event_provided_tickets_override_type;
mod notification_destination_type;
mod order_entry_type;
mod order_type;
mod organization_role_type;
mod organization_type;
mod permission_type;
mod product_type;
mod product_variant_type;
mod room_type;
mod root_site_type;
mod run_type;
mod signup_change_type;
mod signup_request_type;
mod signup_type;
mod staff_position_type;
mod team_member_type;
mod ticket_type;
mod ticket_type_type;
mod user_activity_alert_type;
mod user_con_profile_type;
mod user_type;

pub use ability_type::*;
pub use cms_content_group_type::*;
pub use convention_type::*;
pub use coupon_type::*;
pub use department_type::*;
pub use event_category_type::*;
pub use event_proposal_type::*;
pub use event_type::*;
pub use form_item_type::*;
pub use form_response_change_type::*;
pub use form_section_type::*;
pub use form_type::*;
pub use mailing_lists_type::*;
pub use maximum_event_provided_tickets_override_type::*;
pub use notification_destination_type::*;
pub use order_entry_type::*;
pub use order_type::*;
pub use organization_role_type::*;
pub use organization_type::*;
pub use permission_type::*;
pub use root_site_type::*;
pub use run_type::*;
pub use signup_change_type::*;
pub use signup_request_type::*;
pub use signup_type::*;
pub use staff_position_type::*;
pub use team_member_type::*;
pub use ticket_type::*;
pub use user_activity_alert_type::*;
pub use user_con_profile_type::*;
pub use user_type::*;

#[macro_export]
macro_rules! merged_model_backed_type {
  ($name: ident, $model: path, $graphql_name: expr, $($types: path),+) => {
    #[derive(::async_graphql::MergedObject)]
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

      fn take_arc(self) -> ::std::sync::Arc<Self::Model> {
        self.0.take_arc()
      }
    }
  };
}
