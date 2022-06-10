use serde::{ser::SerializeStruct, Serialize};

use crate::conventions;

impl Serialize for conventions::Model {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let mut state = serializer.serialize_struct("Convention", 3)?;
    state.serialize_field("id", &self.id)?;
    state.serialize_field("name", &self.name)?;
    state.serialize_field("location", &self.location)?;
    state.end()
  }
}
