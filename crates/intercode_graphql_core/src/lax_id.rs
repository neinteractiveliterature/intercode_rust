use std::num::ParseIntError;

use async_graphql::ID;
use once_cell::sync::Lazy;
use regex::Regex;

static REPLACE_TRAILING_STRING_IN_NUMERIC_ID_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new("^(\\d+)\\D.*$").unwrap());

pub struct LaxId(ID);

impl LaxId {
  pub fn parse(id: ID) -> Result<i64, ParseIntError> {
    LaxId(id).try_into()
  }
}

impl From<ID> for LaxId {
  fn from(value: ID) -> Self {
    LaxId(value)
  }
}

impl TryFrom<LaxId> for i64 {
  type Error = ParseIntError;

  fn try_from(value: LaxId) -> Result<Self, Self::Error> {
    let normalized_id = REPLACE_TRAILING_STRING_IN_NUMERIC_ID_REGEX.replace(&value.0 .0, "$1");
    normalized_id.parse()
  }
}
