use async_graphql::*;

pub struct LiquidAssignType {
  name: String,
}

#[Object(name = "LiquidAssign")]
impl LiquidAssignType {
  async fn name(&self) -> &str {
    self.name.as_str()
  }
}
