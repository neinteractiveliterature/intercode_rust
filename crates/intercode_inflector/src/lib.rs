pub use inflector;
use regex::{Captures, Regex, RegexBuilder};
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, collections::HashMap};

#[derive(Deserialize, Serialize)]
pub struct IntercodeInflectorConfig {
  acronym: Vec<String>,
}

pub struct IntercodeInflector {
  acronyms: HashMap<String, String>,
  id_suffix_regex: Regex,
  word_regex: Regex,
  initial_word_character_regex: Regex,
}

impl IntercodeInflector {
  pub fn new() -> Self {
    let id_suffix_regex = Regex::new(r" id$").unwrap();
    let word_regex = RegexBuilder::new(r"([a-z\d]+)")
      .case_insensitive(true)
      .build()
      .expect("Could not parse regex for word replacement");
    let initial_word_character_regex = Regex::new(r"^(\w)").unwrap();

    IntercodeInflector {
      acronyms: HashMap::<String, String>::new(),
      id_suffix_regex,
      word_regex,
      initial_word_character_regex,
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

    let result = self.id_suffix_regex.replace(result, "");

    let result = self
      .word_regex
      .replace_all(result.borrow(), |caps: &Captures| {
        let lowered = caps[1].to_lowercase();
        self.acronymize_if_applicable(&lowered)
      });

    let result = self
      .initial_word_character_regex
      .replace(result.borrow(), |caps: &Captures| caps[1].to_uppercase());

    result.to_string()
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
