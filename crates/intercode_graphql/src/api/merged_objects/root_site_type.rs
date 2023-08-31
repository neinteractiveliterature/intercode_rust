use intercode_cms::api::partial_objects::RootSiteCmsFields;
use intercode_entities::root_sites;

use crate::merged_model_backed_type;

merged_model_backed_type!(
  RootSiteType,
  root_sites::Model,
  "RootSite",
  RootSiteCmsFields
);
