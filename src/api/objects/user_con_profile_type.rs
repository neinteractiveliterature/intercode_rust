use crate::{loaders::ExpectModel, user_con_profiles, SchemaData};
use async_graphql::*;
use pulldown_cmark::{html, Options, Parser};

use super::{ConventionType, ModelBackedType};
pub struct UserConProfileType {
  model: user_con_profiles::Model,
}

impl ModelBackedType<user_con_profiles::Model> for UserConProfileType {
  fn new(model: user_con_profiles::Model) -> Self {
    UserConProfileType { model }
  }
}

#[Object]
impl UserConProfileType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "bio_html")]
  async fn bio_html(&self) -> Option<String> {
    if let Some(bio) = &self.model.bio {
      let mut options = Options::empty();
      options.insert(Options::ENABLE_STRIKETHROUGH);
      options.insert(Options::ENABLE_FOOTNOTES);
      options.insert(Options::ENABLE_SMART_PUNCTUATION);
      options.insert(Options::ENABLE_TABLES);
      let parser = Parser::new_ext(&bio, options);

      let mut html_output = String::new();
      html::push_html(&mut html_output, parser);
      Some(html_output)
    } else {
      None
    }
  }

  #[graphql(name = "bio_name")]
  async fn bio_name(&self) -> String {
    self.model.bio_name()
  }

  async fn convention(&self, ctx: &Context<'_>) -> Result<ConventionType, Error> {
    let loader = &ctx.data::<SchemaData>()?.convention_id_loader;

    let model = loader
      .load_one(self.model.convention_id)
      .await?
      .expect_model()?;

    Ok(ConventionType::new(model))
  }

  #[graphql(name = "gravatar_url")]
  async fn gravatar_url(&self, ctx: &Context<'_>) -> Result<String, Error> {
    if self.model.gravatar_enabled {
      let loader = &ctx.data::<SchemaData>()?.user_id_loader;

      let model = loader.load_one(self.model.user_id).await?.expect_model()?;
      Ok(format!(
        "https://gravatar.com/avatar/{:x}",
        md5::compute(model.email.trim().to_lowercase())
      ))
    } else {
      Ok(format!(
        "https://gravatar.com/avatar/{:x}",
        md5::compute("badrequest")
      ))
    }
  }

  #[graphql(name = "name_inverted")]
  async fn name_inverted(&self) -> String {
    self.model.name_inverted()
  }

  #[graphql(name = "name_without_nickname")]
  async fn name_without_nickname(&self) -> String {
    self.model.name_without_nickname()
  }
}
