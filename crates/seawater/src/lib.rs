mod any_map;
mod associations;
mod connection_wrapper;
mod context;
mod drop_error;
pub mod loaders;
mod model_backed_drop;
mod normalized_drop_cache;
pub mod preloaders;

pub use connection_wrapper::*;
pub use context::*;
pub use drop_error::*;
pub use model_backed_drop::*;
pub use normalized_drop_cache::*;
use once_cell::sync::Lazy;
use regex::Regex;
pub use seawater_derive::*;

static MODULIZED_TYPE_NAME_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"([A-Za-z]\w+::)+(?P<n>[A-Za-z]\w+)").unwrap());

pub fn demodulize_type_name(type_name: &str) -> String {
  MODULIZED_TYPE_NAME_RE
    .replace_all(type_name, "$n")
    .to_string()
}

pub fn pretty_type_name<T>() -> String {
  demodulize_type_name(std::any::type_name::<T>())
}
