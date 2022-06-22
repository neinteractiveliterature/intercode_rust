use super::UserNames;
use crate::user_con_profiles;

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
