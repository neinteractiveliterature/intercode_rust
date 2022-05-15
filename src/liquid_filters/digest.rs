use liquid_core::{
  Display_filter, Filter, FilterReflection, ParseFilter, Result, Runtime, Value, ValueView,
};

use super::invalid_input;

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
  name = "md5",
  description = "Computes the MD5 hash of the input string (not including leading/trailing whitespace) and \
    outputs it in hex format.",
  parsed(MD5Filter)
)]
pub struct MD5;

#[derive(Debug, Default, Display_filter)]
#[name = "md5"]
struct MD5Filter;

impl Filter for MD5Filter {
  fn evaluate(&self, input: &dyn ValueView, _runtime: &dyn Runtime) -> Result<Value> {
    let input = input
      .as_scalar()
      .ok_or_else(|| invalid_input("String expected"))
      .unwrap()
      .to_kstr()
      .into_string();

    let digest = md5::compute(input.trim());
    Ok(Value::scalar(format!("{:x}", digest)))
  }
}
