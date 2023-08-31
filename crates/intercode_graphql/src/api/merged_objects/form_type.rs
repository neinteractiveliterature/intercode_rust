use async_graphql::*;
use intercode_conventions::partial_objects::FormConventionsFields;
use intercode_entities::forms;
use intercode_events::partial_objects::FormEventsFields;
use intercode_forms::partial_objects::FormFormsFields;
use intercode_graphql_core::{model_backed_type, ModelBackedType};

use crate::merged_model_backed_type;

use super::{ConventionType, EventCategoryType};

model_backed_type!(FormGlueFields, forms::Model);

#[Object]
impl FormGlueFields {
  #[graphql(name = "event_categories")]
  async fn event_categories(&self, ctx: &Context<'_>) -> Result<Vec<EventCategoryType>, Error> {
    FormEventsFields::from_type(self.clone())
      .event_categories(ctx)
      .await
      .map(|res| res.into_iter().map(EventCategoryType::from_type).collect())
  }

  #[graphql(name = "proposal_event_categories")]
  async fn proposal_event_categories(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<EventCategoryType>, Error> {
    FormEventsFields::from_type(self.clone())
      .proposal_event_categories(ctx)
      .await
      .map(|res| res.into_iter().map(EventCategoryType::from_type).collect())
  }

  #[graphql(name = "user_con_profile_conventions")]
  async fn user_con_profile_conventions(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<ConventionType>, Error> {
    FormConventionsFields::from_type(self.clone())
      .user_con_profile_conventions(ctx)
      .await
      .map(|res| res.into_iter().map(ConventionType::from_type).collect())
  }
}

merged_model_backed_type!(
  FormType,
  forms::Model,
  "Form",
  FormGlueFields,
  FormFormsFields
);
