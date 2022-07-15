use super::{DropError, SignupDrop};
use intercode_entities::user_con_profiles;
use intercode_graphql::loaders::expect::ExpectModels;
use intercode_graphql::SchemaData;
use intercode_inflector::IntercodeInflector;
use lazy_liquid_value_view::liquid_drop;

#[derive(Debug, Clone)]
pub struct UserConProfileDrop {
  user_con_profile: user_con_profiles::Model,
  schema_data: SchemaData,
  drop_cache: UserConProfileDropCache,
}

#[derive(Debug, Clone, Default)]
struct UserConProfileDropCache {
  id: tokio::sync::OnceCell<liquid::model::Value>,
  first_name: tokio::sync::OnceCell<liquid::model::Value>,
  last_name: tokio::sync::OnceCell<liquid::model::Value>,
  privileges: tokio::sync::OnceCell<liquid::model::Value>,
  signups: tokio::sync::OnceCell<liquid::model::Value>,
}

impl UserConProfileDrop {
  pub fn new(user_con_profile: user_con_profiles::Model, schema_data: SchemaData) -> Self {
    UserConProfileDrop {
      user_con_profile,
      schema_data,
      drop_cache: Default::default(),
    }
  }
  async fn id(&self) -> &dyn liquid::ValueView {
    self
      .drop_cache
      .id
      .get_or_init(|| async move {
        liquid::model::to_value(&self.user_con_profile.id).unwrap_or(liquid::model::Value::Nil)
      })
      .await
      .as_view()
  }
  async fn first_name(&self) -> &dyn liquid::ValueView {
    self
      .drop_cache
      .first_name
      .get_or_init(|| async move {
        liquid::model::to_value(&self.user_con_profile.first_name.as_str())
          .unwrap_or(liquid::model::Value::Nil)
      })
      .await
      .as_view()
  }
  async fn last_name(&self) -> &dyn liquid::ValueView {
    self
      .drop_cache
      .last_name
      .get_or_init(|| async move {
        liquid::model::to_value(&self.user_con_profile.last_name.as_str())
          .unwrap_or(liquid::model::Value::Nil)
      })
      .await
      .as_view()
  }
  async fn privileges(&self) -> &dyn liquid::ValueView {
    self
      .drop_cache
      .privileges
      .get_or_try_init(|| async move {
        let result = self
          .schema_data
          .loaders
          .user_con_profile_user
          .load_one(self.user_con_profile.id)
          .await?;
        let user = result.expect_one()?;
        let inflector = IntercodeInflector::new();
        Ok(
          user
            .privileges()
            .iter()
            .map(|priv_name| inflector.humanize(priv_name))
            .collect::<Vec<_>>(),
        )
        .and_then(|value_convertible| liquid::model::to_value(&value_convertible))
        .map_err(|error| DropError::from(error))
      })
      .await
      .unwrap_or(&liquid::model::Value::Nil)
      .as_view()
  }
  async fn signups(&self) -> &dyn liquid::ValueView {
    self
      .drop_cache
      .signups
      .get_or_try_init(|| async move {
        let result = self
          .schema_data
          .loaders
          .user_con_profile_signups
          .load_one(self.user_con_profile.id)
          .await?;
        let signups = result.expect_models()?;
        liquid::model::to_value(
          &signups
            .iter()
            .map(|signup| SignupDrop::new(signup))
            .collect::<Vec<_>>(),
        )
        .map_err(|err| DropError::from(err))
      })
      .await
      .unwrap_or(&liquid::model::Value::Nil)
      .as_view()
  }
}
impl serde::ser::Serialize for UserConProfileDrop {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::ser::Serializer,
  {
    use serde::ser::SerializeStruct;
    let mut struct_serializer = serializer.serialize_struct("UserConProfileDrop", 5usize)?;
    let (id, first_name, last_name, privileges, signups) = tokio::task::block_in_place(move || {
      tokio::runtime::Handle::current().block_on(async move {
        futures::join!(
          self.id(),
          self.first_name(),
          self.last_name(),
          self.privileges(),
          self.signups()
        )
      })
    });
    struct_serializer.serialize_field("id", &id.to_value());
    struct_serializer.serialize_field("first_name", &first_name.to_value());
    struct_serializer.serialize_field("last_name", &last_name.to_value());
    struct_serializer.serialize_field("privileges", &privileges.to_value());
    struct_serializer.serialize_field("signups", &signups.to_value());
    struct_serializer.end()
  }
}
impl liquid::ValueView for UserConProfileDrop {
  fn as_debug(&self) -> &dyn std::fmt::Debug {
    self as &dyn std::fmt::Debug
  }
  fn render(&self) -> liquid::model::DisplayCow<'_> {
    liquid::model::DisplayCow::Owned(Box::new("UserConProfileDrop"))
  }
  fn source(&self) -> liquid::model::DisplayCow<'_> {
    liquid::model::DisplayCow::Owned(Box::new("UserConProfileDrop"))
  }
  fn type_name(&self) -> &'static str {
    "UserConProfileDrop"
  }
  fn query_state(&self, state: liquid::model::State) -> bool {
    match state {
      liquid::model::State::Truthy => true,
      liquid::model::State::DefaultValue => false,
      liquid::model::State::Empty => false,
      liquid::model::State::Blank => false,
    }
  }
  fn to_kstr(&self) -> liquid::model::KStringCow<'_> {
    "UserConProfileDrop".to_kstr()
  }
  fn to_value(&self) -> liquid_core::Value {
    todo!()
  }
}
impl liquid::ObjectView for UserConProfileDrop {
  fn as_value(&self) -> &dyn liquid::ValueView {
    self as &dyn liquid::ValueView
  }
  fn size(&self) -> i64 {
    usize::try_into(5usize).unwrap()
  }
  fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = liquid::model::KStringCow<'k>> + 'k> {
    Box::new(
      vec!["id", "first_name", "last_name", "privileges", "signups"]
        .into_iter()
        .map(|s| s.into()),
    )
  }
  fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn liquid::ValueView> + 'k> {
    let (id, first_name, last_name, privileges, signups) = tokio::task::block_in_place(move || {
      tokio::runtime::Handle::current().block_on(async move {
        futures::join!(
          self.id(),
          self.first_name(),
          self.last_name(),
          self.privileges(),
          self.signups()
        )
      })
    });
    let values = vec![id, first_name, last_name, privileges, signups];
    Box::new(values.into_iter())
  }
  fn iter<'k>(
    &'k self,
  ) -> Box<dyn Iterator<Item = (liquid::model::KStringCow<'k>, &'k dyn liquid::ValueView)> + 'k> {
    let (id, first_name, last_name, privileges, signups) = tokio::task::block_in_place(move || {
      tokio::runtime::Handle::current().block_on(async move {
        futures::join!(
          self.id(),
          self.first_name(),
          self.last_name(),
          self.privileges(),
          self.signups()
        )
      })
    });
    let pairs = vec![
      ("id", id),
      ("first_name", first_name),
      ("last_name", last_name),
      ("privileges", privileges),
      ("signups", signups),
    ];
    Box::new(pairs.into_iter().map(|(key, value)| (key.into(), value)))
  }
  fn contains_key(&self, index: &str) -> bool {
    match index {
      "id" | "first_name" | "last_name" | "privileges" | "signups" => true,
      _ => false,
    }
  }
  fn get<'s>(&'s self, index: &str) -> Option<&'s dyn liquid::ValueView> {
    tokio::task::block_in_place(move || {
      tokio::runtime::Handle::current().block_on(async move {
        match index {
          "id" => Some(self.id().await),
          "first_name" => Some(self.first_name().await),
          "last_name" => Some(self.last_name().await),
          "privileges" => Some(self.privileges().await),
          "signups" => Some(self.signups().await),
          _ => None,
        }
      })
    })
  }
}
