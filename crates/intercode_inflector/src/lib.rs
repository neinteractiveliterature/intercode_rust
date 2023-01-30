pub use inflector;
use inflector::string::pluralize;
use once_cell::sync::Lazy;
use regex::{Captures, Regex, RegexBuilder};
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, collections::HashMap};

static ID_SUFFIX_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r" id$").unwrap());
static WORD_REGEX: Lazy<Regex> = Lazy::new(|| {
  RegexBuilder::new(r"([a-z\d]+)")
    .case_insensitive(true)
    .build()
    .expect("Could not parse regex for word replacement")
});
static INITIAL_WORD_CHARACTER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\w)").unwrap());

#[derive(Deserialize, Serialize)]
pub struct IntercodeInflectorConfig {
  acronym: Vec<String>,
}

pub struct IntercodeInflector {
  acronyms: HashMap<String, String>,
}

impl IntercodeInflector {
  pub fn new() -> Self {
    IntercodeInflector {
      acronyms: HashMap::<String, String>::new(),
    }
  }

  pub fn with_default_acronyms(mut self) -> Self {
    let inflector_config = include_str!("../../../config/inflections.json");
    let parsed_config: IntercodeInflectorConfig =
      serde_json::from_str(inflector_config).expect("Invalid inflector config!");

    for acronym in parsed_config.acronym {
      self.register_acronym(&acronym);
    }

    self
  }

  pub fn register_acronym(&mut self, acronym: &str) {
    self
      .acronyms
      .insert(acronym.to_lowercase(), acronym.to_string());
  }

  pub fn acronymize_if_applicable(&self, word: &str) -> String {
    let acronym = self.acronyms.get(&word.to_lowercase());

    match acronym {
      None => word.to_string(),
      Some(acronymized_word) => acronymized_word.to_string(),
    }
  }

  pub fn humanize(&self, input: &str) -> String {
    let result = input.replace('_', " ");
    let result = result.trim_start();

    let result = ID_SUFFIX_REGEX.replace(result, "");

    let result = WORD_REGEX.replace_all(result.borrow(), |caps: &Captures| {
      let lowered = caps[1].to_lowercase();
      self.acronymize_if_applicable(&lowered)
    });

    let result = INITIAL_WORD_CHARACTER_REGEX
      .replace(result.borrow(), |caps: &Captures| caps[1].to_uppercase());

    result.to_string()
  }

  pub fn pluralize(&self, input: &str) -> String {
    pluralize::to_plural(input)
  }
}

impl Default for IntercodeInflector {
  fn default() -> Self {
    Self::new().with_default_acronyms()
  }
}

#[cfg(test)]
mod tests {
  use crate::IntercodeInflector;

  #[test]
  fn default_acronyms_work() {
    let inflector = IntercodeInflector::new().with_default_acronyms();

    assert_eq!("GMs", inflector.acronymize_if_applicable("Gms"));
    assert_eq!("GM", inflector.acronymize_if_applicable("gm"));
  }

  #[test]
  fn non_acronyms_dont_change() {
    let inflector = IntercodeInflector::new().with_default_acronyms();

    assert_eq!("Parks", inflector.acronymize_if_applicable("Parks"));
    assert_eq!("and", inflector.acronymize_if_applicable("and"));
    assert_eq!(
      "RECREATION",
      inflector.acronymize_if_applicable("RECREATION")
    );
  }

  #[test]
  fn registering_acronyms_works() {
    let mut inflector = IntercodeInflector::new();

    assert_eq!("tcp/ip", inflector.acronymize_if_applicable("tcp/ip"));
    inflector.register_acronym("TCP/IP");
    assert_eq!("TCP/IP", inflector.acronymize_if_applicable("tcp/ip"));
  }

  mod humanize {
    use crate::IntercodeInflector;

    #[test]
    fn humanize_works_with_acronyms() {
      let inflector = IntercodeInflector::new().with_default_acronyms();

      assert_eq!("Assistant GMs", inflector.humanize("assistant_gms_id"));
    }
  }
}
