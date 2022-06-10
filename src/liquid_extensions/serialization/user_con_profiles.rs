use serde::{ser::SerializeStruct, Serialize};

use crate::user_con_profiles;

impl Serialize for user_con_profiles::Model {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let mut state = serializer.serialize_struct("UserConProfile", 3)?;
    state.serialize_field("id", &self.id)?;
    state.serialize_field("first_name", &self.first_name)?;
    state.serialize_field("last_name", &self.last_name)?;
    state.end()
  }
}
