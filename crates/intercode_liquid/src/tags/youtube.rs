use std::io::Write;

use html_escape::encode_double_quoted_attribute;
use liquid::Error;
use liquid_core::{
  Expression, Language, ParseTag, Renderable, Result, Runtime, TagReflection, TagTokenIter,
  ValueView,
};

#[derive(Copy, Clone, Debug, Default)]
pub struct YouTubeTag;

impl YouTubeTag {
  pub fn new() -> Self {
    Self
  }
}

impl TagReflection for YouTubeTag {
  fn tag(&self) -> &'static str {
    "youtube"
  }

  fn description(&self) -> &'static str {
    "Embeds a YouTube video.  The video ID must be provided."
  }
}

impl ParseTag for YouTubeTag {
  fn parse(
    &self,
    mut arguments: TagTokenIter<'_>,
    _options: &Language,
  ) -> Result<Box<dyn Renderable>> {
    let video_id = arguments.expect_next("Identifier or literal expected.")?;
    let video_id = video_id.expect_value().into_result()?;

    arguments.expect_nothing()?;

    Ok(Box::new(YouTube { video_id }))
  }

  fn reflection(&self) -> &dyn TagReflection {
    self
  }
}

#[derive(Debug)]
struct YouTube {
  video_id: Expression,
}

impl Renderable for YouTube {
  fn render_to(&self, writer: &mut dyn Write, runtime: &dyn Runtime) -> Result<()> {
    let video_id = self.video_id.evaluate(runtime)?;
    if !video_id.is_scalar() {
      return Error::with_msg("video_id must be a string")
        .context("youtube", format!("{}", video_id.source()))
        .into_err();
    }
    let video_id = video_id.to_kstr().into_owned();

    let tag = format!(
      "<iframe type=\"text/html\" width=\"640\" height=\"390\" \
            src=\"https://www.youtube.com/embed/#{}?enablejsapi=1\" \
            frameborder=\"0\"></iframe>",
      encode_double_quoted_attribute(&video_id)
    );

    if let Err(error) = writer.write(tag.as_bytes()) {
      Err(Error::with_msg(error.to_string()))
    } else {
      Ok(())
    }
  }
}
