use crate::user_con_profiles;

impl user_con_profiles::Model {
  pub fn bio_name(self: &Self) -> String {
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

  pub fn name_without_nickname(self: &Self) -> String {
    format!("{} {}", self.first_name, self.last_name)
      .trim()
      .to_string()
  }

  pub fn name_inverted(self: &Self) -> String {
    format!("{}, {}", self.last_name, self.first_name)
      .trim()
      .to_string()
  }
}
