mod entities_by_id_loader;
mod entities_by_link_loader;
mod entities_by_relation_loader;
mod expect;

pub use entities_by_id_loader::*;
pub use entities_by_link_loader::*;
pub use entities_by_relation_loader::*;
pub use expect::*;
use sea_orm::{EntityTrait, ModelTrait, PrimaryKeyTrait};

pub trait AssociationLoaderResult<FromModel: ModelTrait, ToModel: ModelTrait> {
  fn get_from_id(
    &self,
  ) -> <<FromModel::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType;
  fn get_models(&self) -> &Vec<ToModel>;
}
