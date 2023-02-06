use liquid_core::{
  parser::BlockElement, BlockReflection, Language, ParseBlock, Renderable, Result, TagBlock,
  TagTokenIter, Template,
};
use liquid_lib::stdlib::IfBlock;
use once_cell::sync::Lazy;

#[derive(Copy, Clone, Debug, Default)]
pub struct BritaIfBlock;

static DEFAULT_IF_BLOCK: Lazy<IfBlock> = Lazy::new(IfBlock::default);

impl BritaIfBlock {
  pub fn new() -> Self {
    Self::default()
  }
}

impl ParseBlock for BritaIfBlock {
  fn parse(
    &self,
    arguments: TagTokenIter,
    block: TagBlock,
    options: &Language,
  ) -> Result<Box<dyn Renderable>> {
    DEFAULT_IF_BLOCK.parse(arguments, block, options)
  }

  fn reflection(&self) -> &dyn BlockReflection {
    self
  }
}

impl BlockReflection for BritaIfBlock {
  fn start_tag(&self) -> &str {
    DEFAULT_IF_BLOCK.start_tag()
  }

  fn end_tag(&self) -> &str {
    DEFAULT_IF_BLOCK.end_tag()
  }

  fn description(&self) -> &str {
    DEFAULT_IF_BLOCK.description()
  }
}
