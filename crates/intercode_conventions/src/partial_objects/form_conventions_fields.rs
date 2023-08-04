use async_graphql::*;
use intercode_entities::forms;
use intercode_graphql_core::{load_one_by_model_id, loader_result_to_many, model_backed_type};

use super::ConventionConventionsFields;

model_backed_type!(FormConventionsFields, forms::Model);

impl FormConventionsFields {
  pub async fn user_con_profile_conventions(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<ConventionConventionsFields>, Error> {
    let loader_result = load_one_by_model_id!(form_user_con_profile_conventions, ctx, self)?;
    Ok(loader_result_to_many!(
      loader_result,
      ConventionConventionsFields
    ))
  }
}
