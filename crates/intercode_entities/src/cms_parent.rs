use crate::{
  cms_files, cms_graphql_queries, cms_layouts, cms_navigation_items, cms_partials, cms_variables,
  conventions, pages, root_sites,
};
use async_trait::async_trait;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Select};

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

#[async_trait]
pub trait CmsParentTrait {
  fn cms_files(&self) -> Select<cms_files::Entity>;
  fn cms_graphql_queries(&self) -> Select<cms_graphql_queries::Entity>;
  fn cms_layouts(&self) -> Select<cms_layouts::Entity>;
  fn cms_navigation_items(&self) -> Select<cms_navigation_items::Entity>;
  fn cms_partials(&self) -> Select<cms_partials::Entity>;
  fn cms_variables(&self) -> Select<cms_variables::Entity>;
  fn default_layout(&self) -> Select<cms_layouts::Entity>;
  fn pages(&self) -> Select<pages::Entity>;

  fn root_page(&self) -> Select<pages::Entity>;

  fn cms_page_for_path(&self, path: &str) -> Option<Select<pages::Entity>> {
    if path.starts_with("/pages/") {
      let (_, slug) = path.split_at(7);
      Some(self.pages().filter(pages::Column::Slug.eq(slug)))
    } else if path == "/" {
      Some(self.root_page())
    } else {
      None
    }
  }

  async fn effective_cms_layout<C: ConnectionTrait>(
    &self,
    path: &str,
    db: &C,
  ) -> Result<cms_layouts::Model, sea_orm::DbErr> {
    let page_scope = self.cms_page_for_path(path);

    if let Some(page_scope) = page_scope {
      let layout_from_page = page_scope
        .find_also_related(cms_layouts::Entity)
        .one(db)
        .await?
        .and_then(|(_page, layout)| layout);

      if let Some(layout) = layout_from_page {
        return Ok(layout);
      }
    }

    self
      .default_layout()
      .one(db)
      .await?
      .ok_or_else(|| sea_orm::DbErr::RecordNotFound("No default layout found".to_string()))
  }
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
  enum_assoc!(cms_layouts);
  enum_assoc!(cms_navigation_items);
  enum_assoc!(cms_partials);
  enum_assoc!(cms_variables);
  enum_assoc!(pages);

  fn default_layout(&self) -> Select<cms_layouts::Entity> {
    match self {
      CmsParent::Convention(convention) => convention.default_layout(),
      CmsParent::RootSite(root_site) => root_site.default_layout(),
    }
  }

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
  convention_assoc!(cms_layouts);
  convention_assoc!(cms_navigation_items);
  convention_assoc!(cms_partials);
  convention_assoc!(cms_variables);
  convention_assoc!(pages);

  fn default_layout(&self) -> Select<cms_layouts::Entity> {
    cms_layouts::Entity::find().filter(cms_layouts::Column::Id.eq(self.default_layout_id))
  }

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
  root_site_assoc!(cms_layouts);
  root_site_assoc!(cms_navigation_items);
  root_site_assoc!(cms_partials);
  root_site_assoc!(cms_variables);
  root_site_assoc!(pages);

  fn default_layout(&self) -> Select<cms_layouts::Entity> {
    cms_layouts::Entity::find().filter(cms_layouts::Column::Id.eq(self.default_layout_id))
  }

  fn root_page(&self) -> Select<pages::Entity> {
    pages::Entity::find().filter(pages::Column::Id.eq(self.root_page_id))
  }
}
