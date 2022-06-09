use sea_orm::ModelTrait;

pub trait ExpectModel<M: ModelTrait> {
  fn expect_model(self: &Self) -> Result<M, async_graphql::Error>;
}

pub trait ExpectModels<M: ModelTrait> {
  fn expect_models(self: &Self) -> Result<&Vec<M>, async_graphql::Error>;
  fn expect_one(self: &Self) -> Result<&M, async_graphql::Error>;
}
