mod any_store;
mod arc_value_view;
mod connection_wrapper;
mod context;
mod drop_error;
mod drop_ref;
mod drop_result;
mod drop_store;
mod extended_drop_result;
mod liquid_drop;
pub mod loaders;
mod model_backed_drop;
mod optional_value_view;
pub mod preloaders;

pub use arc_value_view::*;
pub use connection_wrapper::*;
pub use context::*;
pub use drop_error::*;
pub use drop_ref::*;
pub use drop_result::*;
pub use drop_store::*;
pub use extended_drop_result::*;
pub use liquid_drop::*;
pub use model_backed_drop::*;
use once_cell::sync::Lazy;
pub use optional_value_view::*;
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
