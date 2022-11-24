use async_graphql::*;
use intercode_entities::cms_variables;
use liquid::ValueView;

pub struct LiquidAssignType {
  name: String,
  drop_class_name: String,
  cms_variable_value: Option<serde_json::Value>,
}

impl LiquidAssignType {
  pub fn from_value_view(name: String, value: &dyn ValueView) -> Self {
    let drop_class_name = value.type_name();

    LiquidAssignType {
      name,
      drop_class_name: drop_class_name.to_string(),
      cms_variable_value: None,
    }
  }

  pub fn from_cms_variable(cms_variable: &cms_variables::Model) -> Self {
    LiquidAssignType {
      name: cms_variable.key.clone(),
      drop_class_name: "CmsVariable".to_string(),
      cms_variable_value: cms_variable.value.clone(),
    }
  }
}

#[Object(name = "LiquidAssign")]
impl LiquidAssignType {
  async fn name(&self) -> &str {
    self.name.as_str()
  }

  #[graphql(name = "drop_class_name")]
  async fn drop_class_name(&self) -> &str {
    self.drop_class_name.as_str()
  }

  #[graphql(name = "cms_variable_value")]
  async fn cms_variable_value(&self) -> Option<&serde_json::Value> {
    self.cms_variable_value.as_ref()
  }
}
