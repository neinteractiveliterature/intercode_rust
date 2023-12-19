use async_graphql::*;
use intercode_conventions::partial_objects::FormConventionsFields;
use intercode_entities::forms;
use intercode_events::partial_objects::FormEventsFields;
use intercode_forms::partial_objects::{FormFormsExtensions, FormFormsFields};
use intercode_graphql_core::{model_backed_type, ModelBackedType};

use crate::merged_model_backed_type;

use super::{ConventionType, EventCategoryType, FormSectionType};

model_backed_type!(FormGlueFields, forms::Model);

impl FormFormsExtensions for FormGlueFields {}

#[Object]
impl FormGlueFields {
  #[graphql(name = "event_categories")]
  async fn event_categories(&self, ctx: &Context<'_>) -> Result<Vec<EventCategoryType>, Error> {
    FormEventsFields::from_type(self.clone())
      .event_categories(ctx)
      .await
      .map(|res| res.into_iter().map(EventCategoryType::from_type).collect())
  }

  #[graphql(name = "form_section")]
  pub async fn form_section(&self, ctx: &Context<'_>, id: ID) -> Result<FormSectionType, Error> {
    FormFormsExtensions::form_section(self, ctx, id).await
  }

  #[graphql(name = "form_sections")]
  pub async fn form_sections(&self, ctx: &Context<'_>) -> Result<Vec<FormSectionType>, Error> {
    FormFormsExtensions::form_sections(self, ctx).await
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
