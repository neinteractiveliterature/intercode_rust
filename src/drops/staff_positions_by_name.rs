use std::collections::HashMap;

use intercode_graphql::{loaders::expect::ExpectModels, SchemaData};
use lazy_liquid_value_view::DropResult;
use liquid::{ObjectView, ValueView};
use regex::Regex;
use tokio::sync::OnceCell;

use super::{DropError, StaffPositionDrop};

fn normalize_staff_position_name(name: &str) -> String {
  Regex::new("\\W")
    .unwrap()
    .replace_all(name.to_lowercase().as_str(), "_")
    .to_string()
}

#[derive(Debug, Clone)]
pub struct StaffPositionsByName<'a> {
  schema_data: SchemaData,
  convention_id: i64,
  staff_positions: OnceCell<HashMap<String, StaffPositionDrop<'a>>>,
}

impl<'a> StaffPositionsByName<'a> {
  pub fn new(schema_data: SchemaData, convention_id: i64) -> Self {
    StaffPositionsByName {
      schema_data,
      convention_id,
      staff_positions: Default::default(),
    }
  }

  async fn query_and_store(&self) -> Result<HashMap<String, StaffPositionDrop<'a>>, DropError> {
    Ok(
      self
        .schema_data
        .loaders
        .convention_staff_positions
        .load_one(self.convention_id)
        .await?
        .expect_models()?
        .iter()
        .map(|model| {
          (
            normalize_staff_position_name(model.name.as_deref().unwrap_or("")),
            StaffPositionDrop::new(model.clone(), self.schema_data.clone()),
          )
        })
        .collect(),
    )
  }

  fn blocking_get_all(&self) -> Result<&HashMap<String, StaffPositionDrop<'a>>, DropError> {
    tokio::task::block_in_place(|| {
      tokio::runtime::Handle::current().block_on(async move {
        self
          .staff_positions
          .get_or_try_init(|| async move { self.query_and_store().await })
          .await
      })
    })
  }
}

impl<'a> ValueView for StaffPositionsByName<'a> {
  fn as_debug(&self) -> &dyn std::fmt::Debug {
    self
  }

  fn as_object(&self) -> Option<&dyn ObjectView> {
    Some(self)
  }

  fn render(&self) -> liquid::model::DisplayCow<'_> {
    liquid::model::DisplayCow::Owned(Box::new("StaffPositionsByName"))
  }

  fn source(&self) -> liquid::model::DisplayCow<'_> {
    liquid::model::DisplayCow::Owned(Box::new("StaffPositionsByName"))
  }

  fn type_name(&self) -> &'static str {
    "StaffPositionsByName"
  }

  fn query_state(&self, state: liquid::model::State) -> bool {
    match state {
      liquid::model::State::Truthy => true,
      liquid::model::State::DefaultValue => false,
      liquid::model::State::Empty => false,
      liquid::model::State::Blank => false,
    }
  }

  fn to_kstr(&self) -> liquid::model::KStringCow<'_> {
    "StaffPositionsByName".to_kstr()
  }

  fn to_value(&self) -> liquid_core::Value {
    unimplemented!()
  }
}

impl<'a> ObjectView for StaffPositionsByName<'a> {
  fn as_value(&self) -> &dyn ValueView {
    self
  }

  fn size(&self) -> i64 {
    self
      .blocking_get_all()
      .map(|staff_positions| staff_positions.size())
      .unwrap_or(0)
  }

  fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = liquid::model::KStringCow<'k>> + 'k> {
    Box::new(self.iter().map(|(key, _value)| key))
  }

  fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn ValueView> + 'k> {
    Box::new(self.iter().map(|(_key, value)| value))
  }

  fn iter<'k>(
    &'k self,
  ) -> Box<dyn Iterator<Item = (liquid::model::KStringCow<'k>, &'k dyn ValueView)> + 'k> {
    self
      .blocking_get_all()
      .map(|staff_positions| {
        Box::new(
          staff_positions
            .iter()
            .map(|(key, value)| (key.into(), value as &dyn ValueView)),
        ) as Box<dyn Iterator<Item = (liquid::model::KStringCow, &dyn ValueView)>>
      })
      .unwrap_or_else(|_err| Box::new(std::iter::empty()))
  }

  fn contains_key(&self, index: &str) -> bool {
    self
      .blocking_get_all()
      .map(|staff_positions| {
        let normalized_index = normalize_staff_position_name(index);
        staff_positions.contains_key(normalized_index.as_str())
      })
      .unwrap_or(false)
  }

  fn get<'s>(&'s self, index: &str) -> Option<&'s dyn ValueView> {
    self
      .blocking_get_all()
      .ok()
      .and_then(|staff_positions| {
        let normalized_index = normalize_staff_position_name(index);
        staff_positions.get(normalized_index.as_str())
      })
      .map(|drop| drop as &dyn ValueView)
  }
}

impl<'a, 'b: 'a> From<StaffPositionsByName<'b>> for DropResult<'a> {
  fn from(value: StaffPositionsByName<'b>) -> Self {
    DropResult::new(value)
  }
}

impl<'a, 'b: 'a> From<&StaffPositionsByName<'b>> for DropResult<'a> {
  fn from(drop: &StaffPositionsByName<'b>) -> Self {
    DropResult::new(drop.clone())
  }
}
