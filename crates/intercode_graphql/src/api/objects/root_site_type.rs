use async_graphql::*;
use intercode_entities::{cms_parent::CmsParentTrait, root_sites};

use crate::{model_backed_type, SchemaData};

use super::{CmsLayoutType, CmsNavigationItemType, ModelBackedType};
model_backed_type!(RootSiteType, root_sites::Model);

#[Object]
impl RootSiteType {
  pub async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "cms_navigation_items")]
  pub async fn cms_navigation_items(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<CmsNavigationItemType>, Error> {
    let schema_data = ctx.data::<SchemaData>()?;

    Ok(
      self
        .model
        .cms_navigation_items()
        .all(schema_data.db.as_ref())
        .await?
        .iter()
        .map(|item| CmsNavigationItemType::new(item.to_owned()))
        .collect(),
    )
  }

  #[graphql(name = "default_layout")]
  pub async fn default_layout(&self, ctx: &Context<'_>) -> Result<CmsLayoutType, Error> {
    let schema_data = ctx.data::<SchemaData>()?;

    self
      .model
      .default_layout()
      .one(schema_data.db.as_ref())
      .await?
      .ok_or_else(|| Error::new("Default layout not found for root site"))
      .map(CmsLayoutType::new)
  }

  #[graphql(name = "site_name")]
  async fn site_name(&self) -> Option<&str> {
    self.model.site_name.as_deref()
  }
}
