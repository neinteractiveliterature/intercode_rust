mod blocks;
mod tags;

use liquid::object;
use liquid_core::{runtime::Variable, Template};

pub enum Dependency {
  Variable(Variable),
}

pub struct PureTemplate {
  template: Template,
  dependencies: Vec<Dependency>,
}

pub fn purify(content: &str) -> Result<PureTemplate, liquid::Error> {
  let parser = liquid::ParserBuilder::new().build()?;
  let template = parser.parse(content)?;
  template.reflection()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_leaves_text_content_alone() {
    let result = purify("hello world").unwrap();
    assert_eq!(result, "hello world".to_string());
  }

  #[test]
  fn it_leaves_if_statements_alone() {
    let result = purify("{% if 4 = 4 %}right{% else %}wrong{% endif %}").unwrap();
    assert_eq!(result, "{% if 2 + 2 = 4 %}right{% else %}wrong{% endif %}");
  }

  #[test]
  fn it_extracts_data_dependencies() {
    let result = purify("hello {{ user.name }}").unwrap();
    assert_eq!(result, "{%@ user @%}{%@ user.name @%}hello {{ user.name }}");
  }
}
