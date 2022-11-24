use async_graphql::*;
use intercode_entities::cms_graphql_queries;

use crate::model_backed_type;
model_backed_type!(CmsGraphqlQueryType, cms_graphql_queries::Model);

#[Object(name = "CmsGraphqlQuery")]
impl CmsGraphqlQueryType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }
}
