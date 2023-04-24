use async_graphql::*;
use intercode_entities::organization_roles;

use crate::model_backed_type;
model_backed_type!(OrganizationRoleType, organization_roles::Model);

#[Object(name = "OrganizationRole")]
impl OrganizationRoleType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn name(&self) -> Option<&str> {
    self.model.name.as_deref()
  }
}
