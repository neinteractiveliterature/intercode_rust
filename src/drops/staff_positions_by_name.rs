use std::collections::HashMap;

use intercode_entities::{conventions, links::ConventionToStaffPositions};
use intercode_graphql::SchemaData;
use lazy_liquid_value_view::DropResult;
use liquid::{ObjectView, ValueView};
use once_cell::race::OnceBox;
use regex::Regex;
use sea_orm::ModelTrait;
use seawater::{DropError, ModelBackedDrop};

use super::StaffPositionDrop;

fn normalize_staff_position_name(name: &str) -> String {
  Regex::new("\\W")
    .unwrap()
    .replace_all(name.to_lowercase().as_str(), "_")
    .to_string()
}

#[derive(Debug)]
pub struct StaffPositionsByName {
  schema_data: SchemaData,
  convention: conventions::Model,
  staff_positions: OnceBox<HashMap<String, StaffPositionDrop>>,
}

impl StaffPositionsByName {
  pub fn new(schema_data: SchemaData, convention: conventions::Model) -> Self {
    StaffPositionsByName {
      schema_data,
      convention,
      staff_positions: Default::default(),
    }
  }

  async fn query_and_store(&self) -> Result<HashMap<String, StaffPositionDrop>, DropError> {
    Ok(
      self
        .convention
        .find_linked(ConventionToStaffPositions)
        .all(self.schema_data.db.as_ref())
        .await?
        .into_iter()
        .map(|model| {
          (
            normalize_staff_position_name(model.name.as_deref().unwrap_or("")),
            StaffPositionDrop::new(model, self.schema_data.clone()),
          )
        })
        .collect(),
    )
  }

  fn blocking_get_all(&self) -> Result<&HashMap<String, StaffPositionDrop>, DropError> {
    self.staff_positions.get_or_try_init(|| {
      tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
          .block_on(async move { self.query_and_store().await.map(Box::new) })
      })
    })
  }
}

impl ValueView for StaffPositionsByName {
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

impl ObjectView for StaffPositionsByName {
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

impl From<StaffPositionsByName> for DropResult<StaffPositionsByName> {
  fn from(value: StaffPositionsByName) -> Self {
    DropResult::new(value)
  }
}
