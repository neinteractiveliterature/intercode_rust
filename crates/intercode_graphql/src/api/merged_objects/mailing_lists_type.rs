use async_graphql::*;
use intercode_entities::conventions;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_reporting::{
  objects::ContactEmailType,
  partial_objects::{waitlists, MailingListsReportingFields, MailingListsWaitlistsResult},
};

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

model_backed_type!(MailingListsGlueFields, conventions::Model);

#[Object]
impl MailingListsGlueFields {
  async fn waitlists(&self, ctx: &Context<'_>) -> Result<Vec<MailingListsWaitlistsResultWrapper>> {
    Ok(
      waitlists(&self.model, ctx)
        .await?
        .into_iter()
        .map(MailingListsWaitlistsResultWrapper)
        .collect(),
    )
  }
}

#[derive(MergedObject)]
#[graphql(name = "MailingLists")]
pub struct MailingListsType(MailingListsReportingFields, MailingListsGlueFields);

impl ModelBackedType for MailingListsType {
  type Model = conventions::Model;

  fn new(model: Self::Model) -> Self {
    Self(
      MailingListsReportingFields::new(model.clone()),
      MailingListsGlueFields::new(model),
    )
  }

  fn get_model(&self) -> &Self::Model {
    self.0.get_model()
  }

  fn into_model(self) -> Self::Model {
    self.0.into_model()
  }
}
