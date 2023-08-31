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
  pub async fn load_emails(
    destinations: impl IntoIterator<Item = &Self>,
    db: &ConnectionWrapper,
  ) -> Result<Vec<String>, DbErr> {
    let mut user_ids: Vec<i64> = Vec::new();
    let mut staff_position_user_con_profile_ids: Vec<i64> = Vec::new();
    let mut emails: Vec<String> = Vec::new();

    for destination in destinations {
      match destination {
        NotificationDestination::UserConProfile(ucp) => user_ids.push(ucp.user_id),
        NotificationDestination::StaffPosition(sp) => match sp.email.as_ref() {
          Some(email) => emails.push(email.to_owned()),
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

  pub async fn load_sms_numbers(
    destinations: impl IntoIterator<Item = &Self>,
    db: &ConnectionWrapper,
  ) -> Result<Vec<String>, DbErr> {
    let mut user_con_profile_ids: Vec<i64> = Vec::new();
    let mut staff_position_user_con_profile_ids: Vec<i64> = Vec::new();
    let mut sms_numbers: Vec<String> = Vec::new();

    for destination in destinations {
      match destination {
        NotificationDestination::UserConProfile(ucp) => user_con_profile_ids.push(ucp.id),
        NotificationDestination::StaffPosition(sp) => {
          staff_position_user_con_profile_ids.push(sp.id)
        }
      }
    }

    sms_numbers.append(
      &mut user_con_profiles::Entity::find()
        .filter(
          Cond::any()
            .add(user_con_profiles::Column::Id.is_in(user_con_profile_ids))
            .add(
              user_con_profiles::Column::Id.in_subquery(
                QuerySelect::query(
                  &mut staff_positions::Entity::find()
                    .filter(staff_positions::Column::Id.is_in(staff_position_user_con_profile_ids))
                    .find_with_linked(StaffPositionToUserConProfiles)
                    .select_only()
                    .column(user_con_profiles::Column::Id),
                )
                .take(),
              ),
            ),
        )
        .all(db)
        .await?
        .into_iter()
        .filter_map(|ucp| {
          if ucp.allow_sms {
            ucp.mobile_phone
          } else {
            None
          }
        })
        .collect::<Vec<_>>(),
    );

    Ok(sms_numbers)
  }
}
