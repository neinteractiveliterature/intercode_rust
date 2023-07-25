use async_graphql::MergedObject;
use intercode_cms::api::partial_objects::RootSiteCmsFields;
use intercode_entities::root_sites;
use intercode_graphql_core::ModelBackedType;

use crate::merged_model_backed_type;

merged_model_backed_type!(
  RootSiteType,
  root_sites::Model,
  "RootSite",
  RootSiteCmsFields
);
