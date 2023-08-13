use i18n_embed::{
  fluent::{fluent_language_loader, FluentLanguageLoader},
  I18nEmbedError, LanguageLoader,
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "i18n"] // path to the compiled localization resources
pub struct Localizations;

pub fn build_language_loader() -> Result<FluentLanguageLoader, I18nEmbedError> {
  let language_loader = fluent_language_loader!();
  language_loader.load_languages(&Localizations, &[language_loader.fallback_language()])?;

  Ok(language_loader)
}
