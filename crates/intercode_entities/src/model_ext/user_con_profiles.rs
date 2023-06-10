use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect, Select};

use super::UserNames;
use crate::{staff_positions, team_members, user_con_profiles};

pub trait BioEligibility {
  fn bio_eligible(self) -> Select<user_con_profiles::Entity>;
}

impl BioEligibility for Select<user_con_profiles::Entity> {
  fn bio_eligible(self) -> Select<user_con_profiles::Entity> {
    self.filter(
      user_con_profiles::Column::Id.in_subquery(
        QuerySelect::query(
          &mut user_con_profiles::Entity::find()
            .left_join(staff_positions::Entity)
            .left_join(team_members::Entity)
            .filter(
              staff_positions::Column::Id
                .is_not_null()
                .or(team_members::Column::Id.is_not_null()),
            )
            .select_only()
            .column(user_con_profiles::Column::Id),
        )
        .take(),
      ),
    )
  }
}

impl user_con_profiles::Model {
  pub fn bio_name(&self) -> String {
    let mut parts = vec![self.first_name.as_str()];

    let nickname_part: String;

    if let Some(show_nickname_in_bio) = self.show_nickname_in_bio {
      if show_nickname_in_bio {
        if let Some(nickname) = &self.nickname {
          if !nickname.trim().is_empty() {
            nickname_part = format!("\"{}\"", nickname);
            parts.push(nickname_part.as_str());
          }
        }
      }
    }

    parts.push(self.last_name.as_str());

    parts
      .into_iter()
      .filter(|part| !part.trim().is_empty())
      .collect::<Vec<&str>>()
      .join(" ")
  }

  pub fn name(&self) -> String {
    if let Some(nickname) = &self.nickname {
      if !nickname.trim().is_empty() {
        return format!("{} \"{}\" {}", self.first_name, nickname, self.last_name);
      }
    }

    self.name_without_nickname()
  }
}

impl UserNames for user_con_profiles::Model {
  fn get_first_name(&self) -> &str {
    self.first_name.as_str()
  }

  fn get_last_name(&self) -> &str {
    self.last_name.as_str()
  }
}
