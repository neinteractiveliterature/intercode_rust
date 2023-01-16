use async_graphql::dataloader::Loader;
use async_session::async_trait;
use sea_orm::DbErr;
use seawater::ConnectionWrapper;
use std::sync::Arc;

use crate::presenters::signup_count_presenter::{
  load_signup_count_data_for_run_ids, RunSignupCounts,
};

pub struct SignupCountLoader {
  db: ConnectionWrapper,
}

impl SignupCountLoader {
  pub fn new(db: ConnectionWrapper) -> Self {
    SignupCountLoader { db }
  }
}

#[async_trait]
impl Loader<i64> for SignupCountLoader {
  type Value = RunSignupCounts;
  type Error = Arc<DbErr>;

  async fn load(
    &self,
    keys: &[i64],
  ) -> Result<std::collections::HashMap<i64, Self::Value>, Self::Error> {
    load_signup_count_data_for_run_ids(self.db.as_ref(), keys.iter().copied())
      .await
      .map_err(Arc::new)
  }
}
