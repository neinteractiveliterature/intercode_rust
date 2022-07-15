use async_trait::async_trait;
pub use lazy_liquid_value_view_derive::{lazy_value_view, liquid_drop_impl, liquid_drop_struct};
use liquid::ValueView;
use serde::Serialize;
use std::fmt::Debug;
use tokio::runtime::Handle;

#[async_trait]
pub trait LazyValueView {
  type Value: Serialize;
  type Error: From<liquid::Error>;

  async fn resolve(&self) -> Result<&Self::Value, Self::Error>;
  fn get_resolved(&self) -> Option<&Self::Value>;

  fn resolve_sync(&self) -> Result<&Self::Value, Self::Error> {
    tokio::task::block_in_place(|| Handle::current().block_on(async move { self.resolve().await }))
  }

  fn as_value_sync(&self) -> Result<liquid::model::Value, Self::Error> {
    Ok(liquid::model::to_value(self.resolve_sync()?)?)
  }
}

impl<V: Serialize, E: From<liquid::Error>> Debug for dyn LazyValueView<Value = V, Error = E> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let liquid_value =
      liquid::model::to_value(&self.get_resolved()).map_err(|_| std::fmt::Error)?;

    f.debug_tuple("LazyValueView")
      .field(liquid_value.as_debug())
      .finish()
  }
}

// #[derive(Debug)]
// pub struct LiquidDrop {
//   type_name: &'static str,
//   values: HashMap<String, Box<dyn Into<dyn liquid::ValueView>>>,
// }

// impl liquid::ValueView for LiquidDrop {
//   fn as_debug(&self) -> &dyn std::fmt::Debug {
//     self as &dyn Debug
//   }

//   fn render(&self) -> liquid::model::DisplayCow<'_> {
//     self.type_name.render()
//   }

//   fn source(&self) -> liquid::model::DisplayCow<'_> {
//     self.type_name.source()
//   }

//   fn type_name(&self) -> &'static str {
//     self.type_name
//   }

//   fn query_state(&self, state: liquid::model::State) -> bool {
//     match state {
//       liquid::model::State::Truthy => true,
//       liquid::model::State::DefaultValue => false,
//       liquid::model::State::Empty => false,
//       liquid::model::State::Blank => false,
//     }
//   }

//   fn to_kstr(&self) -> liquid::model::KStringCow<'_> {
//     self.type_name.into()
//   }

//   fn to_value(&self) -> liquid_core::Value {
//     let coerced: HashMap<&str, liquid_core::Value> = self
//       .values
//       .iter()
//       .map(|(key, value)| (key.as_str(), value.to_value()))
//       .collect();

//     liquid::model::to_value(&coerced).unwrap()
//   }
// }

// impl liquid::ObjectView for LiquidDrop {
//   fn as_value(&self) -> &dyn ValueView {
//     self as &dyn ValueView
//   }

//   fn size(&self) -> i64 {
//     self.values.len().try_into().unwrap()
//   }

//   fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = liquid::model::KStringCow<'k>> + 'k> {
//     Box::new(self.values.keys().into_iter().map(|key| key.into()))
//   }

//   fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn ValueView> + 'k> {
//     Box::new(self.values.values().into_iter().map(|value| value.as_ref()))
//   }

//   fn iter<'k>(
//     &'k self,
//   ) -> Box<dyn Iterator<Item = (liquid::model::KStringCow<'k>, &'k dyn ValueView)> + 'k> {
//     Box::new(
//       self
//         .values
//         .iter()
//         .map(|(key, value)| (key.into(), value.as_ref())),
//     )
//   }

//   fn contains_key(&self, index: &str) -> bool {
//     self.values.contains_key(index)
//   }

//   fn get<'s>(&'s self, index: &str) -> Option<&'s dyn ValueView> {
//     self.values.get(index).map(|value| value.as_ref())
//   }
// }
