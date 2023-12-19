use async_graphql::*;
use intercode_entities::conventions;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_reporting::{
  objects::ContactEmailType,
  partial_objects::{waitlists, MailingListsReportingFields, MailingListsWaitlistsResult},
};

use crate::merged_model_backed_type;

use super::run_type::RunType;

struct MailingListsWaitlistsResultWrapper(MailingListsWaitlistsResult);

#[Object(name = "MailingListsWaitlistsResult")]
impl MailingListsWaitlistsResultWrapper {
  async fn emails(&self) -> &Vec<ContactEmailType> {
    &self.0.emails
  }

  #[graphql(name = "metadata_fields")]
  async fn metadata_fields(&self) -> &'static [&'static str] {
    &[]
  }

  async fn run(&self) -> RunType {
    RunType::new(self.0.run.clone())
  }
}

impl From<MailingListsWaitlistsResult> for MailingListsWaitlistsResultWrapper {
  fn from(value: MailingListsWaitlistsResult) -> Self {
    Self(value)
  }
}

model_backed_type!(MailingListsGlueFields, conventions::Model);

#[Object]
impl MailingListsGlueFields {
  async fn waitlists(&self, ctx: &Context<'_>) -> Result<Vec<MailingListsWaitlistsResultWrapper>> {
    waitlists(&self.model, ctx).await
  }
}

merged_model_backed_type!(
  MailingListsType,
  conventions::Model,
  "MailingLists",
  MailingListsReportingFields,
  MailingListsGlueFields
);
