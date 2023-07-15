use async_graphql::MergedObject;
use intercode_cms::api::partial_objects::RootSiteCmsFields;
use intercode_entities::root_sites;
use intercode_graphql_core::ModelBackedType;

#[derive(MergedObject)]
#[graphql(name = "RootSite")]
pub struct RootSiteType(RootSiteCmsFields);

impl ModelBackedType for RootSiteType {
  type Model = root_sites::Model;

  fn new(model: Self::Model) -> Self {
    RootSiteType(RootSiteCmsFields::new(model))
  }

  fn get_model(&self) -> &Self::Model {
    self.0.get_model()
  }

  fn into_model(self) -> Self::Model {
    self.0.into_model()
  }
}
