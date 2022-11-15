use crate::cms_navigation_items;
use sea_orm::{Linked, RelationDef, RelationTrait};

#[derive(Debug, Clone)]
pub struct CmsNavigationItemToCmsNavigationSection;

impl Linked for CmsNavigationItemToCmsNavigationSection {
  type FromEntity = cms_navigation_items::Entity;
  type ToEntity = cms_navigation_items::Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![cms_navigation_items::Relation::SelfRef.def()]
  }
}
