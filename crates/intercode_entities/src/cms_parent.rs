use crate::{cms_files, cms_graphql_queries, cms_partials, conventions, pages, root_sites};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Select};

#[derive(Clone, Debug)]
pub enum CmsParent {
  Convention(Box<conventions::Model>),
  RootSite(Box<root_sites::Model>),
}

impl From<conventions::Model> for CmsParent {
  fn from(convention: conventions::Model) -> Self {
    CmsParent::Convention(Box::new(convention))
  }
}

impl From<root_sites::Model> for CmsParent {
  fn from(root_site: root_sites::Model) -> Self {
    CmsParent::RootSite(Box::new(root_site))
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
