use async_graphql::*;
use intercode_entities::root_sites;

use crate::{api::interfaces::CmsParentImplementation, model_backed_type};

model_backed_type!(RootSiteType, root_sites::Model);

#[Object]
impl RootSiteType {
  pub async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "site_name")]
  async fn site_name(&self) -> Option<&str> {
    self.model.site_name.as_deref()
  }
}

impl CmsParentImplementation<root_sites::Model> for RootSiteType {}
