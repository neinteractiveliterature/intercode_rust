use sea_orm::{DbErr, FromQueryResult, QueryResult, TryGetable};

#[derive(Debug)]
pub struct ParentModelIdOnly<ID> {
  pub parent_model_id: ID,
}

impl<ID: TryGetable> FromQueryResult for ParentModelIdOnly<ID> {
  fn from_query_result(res: &QueryResult, pre: &str) -> Result<Self, DbErr> {
    Ok(Self {
      parent_model_id: res.try_get(pre, "parent_model_id")?,
    })
  }
}
