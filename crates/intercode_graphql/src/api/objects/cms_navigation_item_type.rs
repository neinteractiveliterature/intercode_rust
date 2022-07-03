use async_graphql::*;
use intercode_entities::cms_navigation_items;

use crate::{loaders::expect::ExpectModels, model_backed_type, SchemaData};

use super::{ModelBackedType, PageType};
model_backed_type!(CmsNavigationItemType, cms_navigation_items::Model);

#[Object]
impl CmsNavigationItemType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "navigation_section")]
  async fn navigation_section(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<CmsNavigationItemType>, Error> {
    let schema_data = ctx.data::<SchemaData>()?;

    Ok(
      schema_data
        .loaders
        .cms_navigation_item_section
        .load_one(self.model.id)
        .await?
        .try_one()
        .map(|item| CmsNavigationItemType::new(item.to_owned())),
    )
  }

  async fn page(&self, ctx: &Context<'_>) -> Result<Option<PageType>, Error> {
    let schema_data = ctx.data::<SchemaData>()?;

    Ok(
      schema_data
        .loaders
        .cms_navigation_item_page
        .load_one(self.model.id)
        .await?
        .try_one()
        .map(|page| PageType::new(page.to_owned())),
    )
  }

  async fn position(&self) -> Option<i32> {
    self.model.position
  }

  async fn title(&self) -> Option<&str> {
    self.model.title.as_deref()
  }
}
