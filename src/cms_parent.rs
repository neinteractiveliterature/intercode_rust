use std::sync::Arc;

use liquid::partials::InMemorySource;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Select};

use crate::{cms_files, cms_graphql_queries, cms_partials, conventions, pages, root_sites};

#[derive(Clone, Debug)]
pub enum CmsParent {
  Convention(conventions::Model),
  RootSite(root_sites::Model),
}

impl Into<CmsParent> for conventions::Model {
  fn into(self) -> CmsParent {
    CmsParent::Convention(self)
  }
}

impl Into<CmsParent> for root_sites::Model {
  fn into(self) -> CmsParent {
    CmsParent::RootSite(self)
  }
}

pub trait CmsParentTrait {
  fn cms_files(&self) -> Select<cms_files::Entity>;
  fn cms_graphql_queries(&self) -> Select<cms_graphql_queries::Entity>;
  fn cms_partials(&self) -> Select<cms_partials::Entity>;
  fn pages(&self) -> Select<pages::Entity>;

  fn root_page(&self) -> Select<pages::Entity>;
}

macro_rules! enum_assoc {
  ( $x:ident ) => {
    fn $x(&self) -> Select<$x::Entity> {
      match self {
        CmsParent::Convention(convention) => convention.$x(),
        CmsParent::RootSite(root_site) => root_site.$x(),
      }
    }
  };
}

impl CmsParent {
  pub async fn cms_partial_source(
    &self,
    db: Arc<DatabaseConnection>,
  ) -> Result<InMemorySource, sea_orm::DbErr> {
    let mut source = InMemorySource::new();
    let partials = self.cms_partials().all(db.as_ref()).await?;

    for partial in partials.into_iter() {
      source.add(
        partial.name,
        partial.content.unwrap_or_else(|| "".to_string()),
      );
    }

    Ok(source)
  }
}

impl CmsParentTrait for CmsParent {
  enum_assoc!(cms_files);
  enum_assoc!(cms_graphql_queries);
  enum_assoc!(cms_partials);
  enum_assoc!(pages);

  fn root_page(&self) -> Select<pages::Entity> {
    match self {
      CmsParent::Convention(convention) => convention.root_page(),
      CmsParent::RootSite(root_site) => root_site.root_page(),
    }
  }
}

macro_rules! convention_assoc {
  ( $x:ident  ) => {
    fn $x(&self) -> Select<$x::Entity> {
      $x::Entity::find()
        .filter($x::Column::ParentType.eq("Convention"))
        .filter($x::Column::ParentId.eq(self.id))
    }
  };
}

impl CmsParentTrait for conventions::Model {
  convention_assoc!(cms_files);
  convention_assoc!(cms_graphql_queries);
  convention_assoc!(cms_partials);
  convention_assoc!(pages);

  fn root_page(&self) -> Select<pages::Entity> {
    pages::Entity::find().filter(pages::Column::Id.eq(self.root_page_id))
  }
}

macro_rules! root_site_assoc {
  ( $x:ident ) => {
    fn $x(&self) -> Select<$x::Entity> {
      $x::Entity::find()
        .filter($x::Column::ParentType.is_null())
        .filter($x::Column::ParentId.is_null())
    }
  };
}

impl CmsParentTrait for root_sites::Model {
  root_site_assoc!(cms_files);
  root_site_assoc!(cms_graphql_queries);
  root_site_assoc!(cms_partials);
  root_site_assoc!(pages);

  fn root_page(&self) -> Select<pages::Entity> {
    pages::Entity::find().filter(pages::Column::Id.eq(self.root_page_id))
  }
}
