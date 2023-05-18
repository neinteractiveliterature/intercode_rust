use sea_orm::{
  sea_query::{Expr, IntoCondition},
  EntityTrait, Linked, Related,
};

use crate::{notification_destinations, user_activity_alerts};

impl Related<user_activity_alerts::Entity> for notification_destinations::Entity {
  fn to() -> sea_orm::RelationDef {
    notification_destinations::Entity::belongs_to(user_activity_alerts::Entity)
      .from(notification_destinations::Column::SourceId)
      .to(user_activity_alerts::Column::Id)
      .on_condition(|left, _right| {
        Expr::col((left, notification_destinations::Column::SourceType))
          .eq("UserActivityAlert")
          .into_condition()
      })
      .into()
  }
}

impl Related<notification_destinations::Entity> for user_activity_alerts::Entity {
  fn to() -> sea_orm::RelationDef {
    user_activity_alerts::Entity::has_many(notification_destinations::Entity)
      .from(user_activity_alerts::Column::Id)
      .to(notification_destinations::Column::SourceId)
      .on_condition(|_left, right| {
        Expr::col((right, notification_destinations::Column::SourceType))
          .eq("UserActivityAlert")
          .into_condition()
      })
      .into()
  }
}

#[derive(Debug, Clone)]
pub struct UserActivityAlertToNotificationDestinations;

impl Linked for UserActivityAlertToNotificationDestinations {
  type FromEntity = user_activity_alerts::Entity;
  type ToEntity = notification_destinations::Entity;

  fn link(&self) -> Vec<sea_orm::LinkDef> {
    vec![<user_activity_alerts::Entity as Related<
      notification_destinations::Entity,
    >>::to()]
  }
}
