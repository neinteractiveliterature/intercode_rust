pub trait ExpectModel<M> {
  fn expect_model(&self) -> Result<M, async_graphql::Error>;
}

pub trait ExpectModels<M> {
  fn expect_models(&self) -> Result<&Vec<M>, async_graphql::Error>;
  fn expect_one(&self) -> Result<&M, async_graphql::Error>;
  fn try_one(&self) -> Option<&M>;
}
