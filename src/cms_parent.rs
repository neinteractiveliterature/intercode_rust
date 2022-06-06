use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Select};

use crate::{cms_files, cms_graphql_queries, conventions, pages, root_sites};

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
  fn pages(&self) -> Select<pages::Entity>;
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
  enum_assoc!(pages);
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
  convention_assoc!(pages);
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
  root_site_assoc!(pages);
}
