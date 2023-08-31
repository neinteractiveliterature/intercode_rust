use async_graphql::*;
use intercode_entities::forms;
use intercode_graphql_core::{load_one_by_model_id, loader_result_to_many, model_backed_type};

use super::EventCategoryEventsFields;

model_backed_type!(FormEventsFields, forms::Model);

impl FormEventsFields {
  pub async fn event_categories(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<EventCategoryEventsFields>, Error> {
    let loader_result = load_one_by_model_id!(form_event_categories, ctx, self)?;
    Ok(loader_result_to_many!(
      loader_result,
      EventCategoryEventsFields
    ))
  }

  pub async fn proposal_event_categories(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<EventCategoryEventsFields>, Error> {
    let loader_result = load_one_by_model_id!(form_proposal_event_categories, ctx, self)?;
    Ok(loader_result_to_many!(
      loader_result,
      EventCategoryEventsFields
    ))
  }
}
