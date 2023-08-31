use std::io::Write;

use liquid::Error;
use liquid_core::{
  Expression, Language, ParseTag, Renderable, Result, Runtime, TagReflection, TagTokenIter,
  ValueView,
};
use serde_json::json;

use crate::dig::get_array_from_value;
use crate::invalid_argument;

use super::write_react_component_tag;

#[derive(Clone, Debug, Default)]
pub struct MapTag;

impl MapTag {
  pub fn new() -> Self {
    Self
  }
}

impl TagReflection for MapTag {
  fn tag(&self) -> &'static str {
    "map"
  }

  fn description(&self) -> &'static str {
    "Renders a map centered on a given location.  You can pass an explicit height as a CSS length value.  If no \
    height is passed, the height will default to \"30rem\"."
  }
}

impl ParseTag for MapTag {
  fn parse(
    &self,
    mut arguments: TagTokenIter<'_>,
    _options: &Language,
  ) -> Result<Box<dyn Renderable>> {
    let location = arguments
      .expect_next("map requires a location object")?
      .expect_value()
      .into_result()?;

    let height_arg = arguments.next();
    let height = if let Some(height_arg) = height_arg {
      Some(height_arg.expect_value().into_result()?)
    } else {
      None
    };

    arguments.expect_nothing()?;

    Ok(Box::new(Map { height, location }))
  }

  fn reflection(&self) -> &dyn TagReflection {
    self
  }
}

#[derive(Debug)]
struct Map {
  location: Expression,
  height: Option<Expression>,
}

impl Renderable for Map {
  fn render_to(&self, writer: &mut dyn Write, runtime: &dyn Runtime) -> Result<()> {
    let location = self.location.evaluate(runtime)?;
    let location_source = format!("{}", location.source());
    let location = location.as_object().ok_or_else(|| {
      Error::with_msg("location must be an object").context("map", &location_source)
    })?;

    let center = get_array_from_value(&location, "center", "map", &location_source)?;
    let center = center
      .values()
      .map(|value| {
        value
          .as_scalar()
          .and_then(|s| s.to_float())
          .ok_or_else(|| invalid_argument("center", "must be an array of numbers"))
      })
      .collect::<Result<Vec<f64>>>()?;

    let height = self
      .height
      .as_ref()
      .map_or(Ok(None), |expr| {
        let scalar = expr.evaluate(runtime)?;
        let scalar = scalar
          .as_scalar()
          .ok_or_else(|| invalid_argument("height", "must be a string or nil"))?;

        if scalar.is_nil() {
          Ok(None)
        } else {
          Ok(Some(scalar.into_string().to_string()))
        }
      })?
      .unwrap_or_else(|| String::from("30rem"));

    write_react_component_tag(
      writer,
      "Map",
      json!({ "height": height, "center": center, "markerLocation": center }),
    )
  }
}
