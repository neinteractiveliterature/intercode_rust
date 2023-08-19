use intercode_entities::{
  links::StaffPositionToUserConProfiles, staff_positions, user_con_profiles, users,
};
use sea_orm::{sea_query::Cond, ColumnTrait, DbErr, EntityTrait, QueryFilter, QuerySelect};
use seawater::ConnectionWrapper;

pub enum NotificationDestination {
  UserConProfile(user_con_profiles::Model),
  StaffPosition(staff_positions::Model),
}

impl NotificationDestination {
  async fn load_emails(
    destinations: Vec<Self>,
    db: &ConnectionWrapper,
  ) -> Result<Vec<String>, DbErr> {
    let mut user_ids: Vec<i64> = Vec::new();
    let mut staff_position_user_con_profile_ids: Vec<i64> = Vec::new();
    let mut emails: Vec<String> = Vec::with_capacity(destinations.len());

    for destination in destinations {
      match destination {
        NotificationDestination::UserConProfile(ucp) => user_ids.push(ucp.user_id),
        NotificationDestination::StaffPosition(sp) => match sp.email {
          Some(email) => emails.push(email),
          None => staff_position_user_con_profile_ids.push(sp.id),
        },
      }
    }

    emails.append(
      &mut users::Entity::find()
        .filter(
          Cond::any().add(users::Column::Id.is_in(user_ids)).add(
            users::Column::Id.in_subquery(
              QuerySelect::query(
                &mut staff_positions::Entity::find()
                  .filter(staff_positions::Column::Id.is_in(staff_position_user_con_profile_ids))
                  .find_with_linked(StaffPositionToUserConProfiles)
                  .select_only()
                  .column(user_con_profiles::Column::UserId),
              )
              .take(),
            ),
          ),
        )
        .all(db)
        .await?
        .into_iter()
        .map(|user| user.email)
        .collect::<Vec<_>>(),
    );

    Ok(emails)
  }
}
