use async_graphql::*;

pub struct AbilityType;

// TODO just about everything here
#[Object]
impl AbilityType {
  #[graphql(name = "can_manage_conventions")]
  async fn can_manage_conventions(&self) -> bool {
    false
  }

  #[graphql(name = "can_read_schedule")]
  async fn can_read_schedule(&self) -> bool {
    false
  }
  #[graphql(name = "can_read_schedule_with_counts")]
  async fn can_read_schedule_with_counts(&self) -> bool {
    false
  }
  #[graphql(name = "can_list_events")]
  async fn can_list_events(&self) -> bool {
    false
  }
  #[graphql(name = "can_read_user_con_profiles")]
  async fn can_read_user_con_profiles(&self) -> bool {
    false
  }
  #[graphql(name = "can_update_convention")]
  async fn can_update_convention(&self) -> bool {
    false
  }
  #[graphql(name = "can_update_departments")]
  async fn can_update_departments(&self) -> bool {
    false
  }
  #[graphql(name = "can_manage_email_routes")]
  async fn can_manage_email_routes(&self) -> bool {
    false
  }
  #[graphql(name = "can_update_event_categories")]
  async fn can_update_event_categories(&self) -> bool {
    false
  }
  #[graphql(name = "can_read_event_proposals")]
  async fn can_read_event_proposals(&self) -> bool {
    false
  }
  #[graphql(name = "can_manage_runs")]
  async fn can_manage_runs(&self) -> bool {
    false
  }
  #[graphql(name = "can_manage_forms")]
  async fn can_manage_forms(&self) -> bool {
    false
  }
  #[graphql(name = "can_read_any_mailing_list")]
  async fn can_read_any_mailing_list(&self) -> bool {
    false
  }
  #[graphql(name = "can_update_notification_templates")]
  async fn can_update_notification_templates(&self) -> bool {
    false
  }
  #[graphql(name = "can_manage_oauth_applications")]
  async fn can_manage_oauth_applications(&self) -> bool {
    false
  }
  #[graphql(name = "can_read_reports")]
  async fn can_read_reports(&self) -> bool {
    false
  }
  #[graphql(name = "can_manage_rooms")]
  async fn can_manage_rooms(&self) -> bool {
    false
  }
  #[graphql(name = "can_manage_signups")]
  async fn can_manage_signups(&self) -> bool {
    false
  }
  #[graphql(name = "can_manage_any_cms_content")]
  async fn can_manage_any_cms_content(&self) -> bool {
    false
  }
  #[graphql(name = "can_manage_staff_positions")]
  async fn can_manage_staff_positions(&self) -> bool {
    false
  }
  #[graphql(name = "can_read_orders")]
  async fn can_read_orders(&self) -> bool {
    false
  }
  #[graphql(name = "can_manage_ticket_types")]
  async fn can_manage_ticket_types(&self) -> bool {
    false
  }
  #[graphql(name = "can_read_user_activity_alerts")]
  async fn can_read_user_activity_alerts(&self) -> bool {
    false
  }
  #[graphql(name = "can_read_organizations")]
  async fn can_read_organizations(&self) -> bool {
    false
  }
  #[graphql(name = "can_read_users")]
  async fn can_read_users(&self) -> bool {
    false
  }
}
