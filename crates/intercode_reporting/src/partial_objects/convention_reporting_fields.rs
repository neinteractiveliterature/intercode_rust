use intercode_entities::conventions;
use intercode_graphql_core::{model_backed_type, ModelBackedType};

use super::MailingListsReportingFields;

model_backed_type!(ConventionReportingFields, conventions::Model);

impl ConventionReportingFields {
  pub async fn mailing_lists(&self) -> MailingListsReportingFields {
    MailingListsReportingFields::from_type(self.clone())
  }
}
