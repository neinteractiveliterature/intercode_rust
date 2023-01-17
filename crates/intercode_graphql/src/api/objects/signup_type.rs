use async_graphql::*;
use intercode_entities::signups;

use crate::{model_backed_type, QueryData};

model_backed_type!(SignupType, signups::Model);

#[Object(name = "Signup")]
impl SignupType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn state(&self) -> &str {
    &self.model.state
  }

  #[graphql(name = "waitlist_position")]
  async fn waitlist_position(&self, ctx: &Context<'_>) -> Result<Option<usize>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    Ok(
      query_data
        .loaders
        .signup_waitlist_position
        .load_one(self.model.clone().into())
        .await?
        .flatten(),
    )
  }
}
