use async_graphql::{Context, Object, Result};
use intercode_users::mutations::{AcceptClickwrapAgreement, AcceptClickwrapAgreementPayload};

use super::merged_objects::UserConProfileType;

pub struct MutationRoot;

#[Object(name = "Mutation")]
impl MutationRoot {
  pub async fn accept_clickwrap_agreement(
    &self,
    ctx: &Context<'_>,
  ) -> Result<AcceptClickwrapAgreementPayload<UserConProfileType>> {
    AcceptClickwrapAgreement::accept_clickwrap_agreement(ctx).await
  }
}
