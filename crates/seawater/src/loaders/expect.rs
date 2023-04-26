pub trait ExpectModel<M> {
  fn try_one(&self) -> Option<&M>;
  fn expect_one(&self) -> Result<&M, async_graphql::Error>;
}

pub trait ExpectModels<M>: ExpectModel<M> {
  fn expect_models(&self) -> Result<&Vec<M>, async_graphql::Error>;
}
