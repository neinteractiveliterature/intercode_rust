use std::io::Write;

use html_escape::encode_double_quoted_attribute;
use liquid::Error;
use liquid_core::Result;
use serde_json::{Map, Value};

pub fn react_component_tag(component_name: &str, props: Value) -> String {
  let mut component_props = Map::<String, Value>::new();
  component_props.insert(
    String::from("recaptchaSiteKey"),
    Value::String(std::env::var("RECAPTCHA_SITE_KEY").unwrap_or(String::from(""))),
  );
  component_props.insert(
    String::from("mapboxAccessToken"),
    Value::String(std::env::var("MAPBOX_ACCESS_TOKEN").unwrap_or(String::from(""))),
  );

  if let Value::Object(props) = props {
    component_props.extend(props.into_iter());
  }

  format!(
    "<div data-react-class=\"{}\" data-react-props=\"{}\"></div>",
    encode_double_quoted_attribute(component_name),
    encode_double_quoted_attribute(&Value::Object(component_props).to_string())
  )
}

pub fn write_react_component_tag(
  writer: &mut dyn Write,
  component_name: &str,
  props: Value,
) -> Result<()> {
  let tag = react_component_tag(component_name, props);

  if let Err(error) = writer.write(tag.as_bytes()) {
    Err(Error::with_msg(error.to_string()))
  } else {
    Ok(())
  }
}
