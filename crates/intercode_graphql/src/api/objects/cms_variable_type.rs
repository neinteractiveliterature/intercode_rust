use async_graphql::*;
use intercode_entities::cms_variables;

use crate::model_backed_type;
model_backed_type!(CmsVariableType, cms_variables::Model);

#[Object(name = "CmsVariable")]
impl CmsVariableType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }
}
