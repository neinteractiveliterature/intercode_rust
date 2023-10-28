use sea_orm::{Linked, RelationDef, RelationTrait};

use crate::{cms_partials, cms_partials_pages, pages};

#[derive(Debug, Clone)]
pub struct PageToCmsPartials;

impl Linked for PageToCmsPartials {
  type FromEntity = pages::Entity;
  type ToEntity = cms_partials::Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![
      cms_partials_pages::Relation::Pages.def().rev(),
      cms_partials_pages::Relation::CmsPartials.def(),
    ]
  }
}
