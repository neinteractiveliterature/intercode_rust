use std::io::Write;

use liquid_core::{
  BlockReflection, Language, ParseBlock, Renderable, Result, Runtime, TagBlock, TagTokenIter,
};
use serde_json::json;

use super::super::react_component_tag::write_react_component_tag;

#[derive(Copy, Clone, Debug, Default)]
pub struct SpoilerBlock;

impl SpoilerBlock {
  pub fn new() -> Self {
    Self
  }
}

impl BlockReflection for SpoilerBlock {
  fn start_tag(&self) -> &'static str {
    "spoiler"
  }

  fn end_tag(&self) -> &'static str {
    "endspoiler"
  }

  fn description(&self) -> &'static str {
    "Renders the included text as a spoiler (click to reveal)."
  }
}

impl ParseBlock for SpoilerBlock {
  fn parse(
    &self,
    mut arguments: TagTokenIter<'_>,
    mut tokens: TagBlock<'_, '_>,
    options: &Language,
  ) -> Result<Box<dyn Renderable>> {
    arguments.expect_nothing()?;

    let content = tokens.parse_all(options)?;

    Ok(Box::new(Spoiler { content }))
  }

  fn reflection(&self) -> &dyn BlockReflection {
    self
  }
}

#[derive(Debug)]
struct Spoiler {
  content: Vec<Box<dyn Renderable>>,
}

impl Renderable for Spoiler {
  fn render_to(&self, writer: &mut dyn Write, runtime: &dyn Runtime) -> Result<()> {
    let rendered_content: String = self
      .content
      .iter()
      .map(|renderable| renderable.render(runtime))
      .collect::<Result<String>>()?;

    write_react_component_tag(writer, "Spoiler", json!({ "content": rendered_content }))
  }
}
