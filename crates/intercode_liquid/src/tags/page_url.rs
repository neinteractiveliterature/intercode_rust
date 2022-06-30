use std::io::Write;

use liquid::Error;
use liquid_core::{
  Expression, Language, ParseTag, Renderable, Result, Runtime, TagReflection, TagTokenIter,
  ValueView,
};

#[derive(Clone, Debug)]
pub struct PageUrlTag;

impl TagReflection for PageUrlTag {
  fn tag(&self) -> &'static str {
    "page_url"
  }

  fn description(&self) -> &'static str {
    "Given the slug of a page, gives the URL of that page."
  }
}

impl ParseTag for PageUrlTag {
  fn parse(
    &self,
    mut arguments: TagTokenIter<'_>,
    _options: &Language,
  ) -> Result<Box<dyn Renderable>> {
    let slug = arguments.expect_next("Identifier or literal expected.")?;
    let slug = slug.expect_value().into_result()?;

    arguments.expect_nothing()?;

    Ok(Box::new(PageUrl { slug }))
  }

  fn reflection(&self) -> &dyn TagReflection {
    self
  }
}

#[derive(Debug)]
struct PageUrl {
  slug: Expression,
}

impl Renderable for PageUrl {
  fn render_to(&self, writer: &mut dyn Write, runtime: &dyn Runtime) -> Result<()> {
    let slug = self.slug.evaluate(runtime)?;
    if !slug.is_scalar() {
      return Error::with_msg("slug must be a string")
        .context("page_url", format!("{}", slug.source()))
        .into_err();
    }
    let slug = slug.to_kstr().into_owned();

    // TODO do something actually real here
    let url = format!("/pages/{}", slug);

    if let Err(error) = writer.write(url.as_bytes()) {
      Err(Error::with_msg(error.to_string()))
    } else {
      Ok(())
    }
  }
}
