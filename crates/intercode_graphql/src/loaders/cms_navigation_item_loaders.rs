use intercode_entities::{cms_navigation_items, pages};
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

impl_to_entity_link_loader!(
  cms_navigation_items::Entity,
  CmsNavigationItemToCmsNavigationSection,
  cms_navigation_items::Entity,
  cms_navigation_items::PrimaryKey::Id
);

impl_to_entity_relation_loader!(
  cms_navigation_items::Entity,
  pages::Entity,
  cms_navigation_items::PrimaryKey::Id
);
