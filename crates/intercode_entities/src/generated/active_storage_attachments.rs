//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "active_storage_attachments")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub name: String,
  pub record_type: String,
  pub record_id: i64,
  pub blob_id: i64,
  pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::active_storage_blobs::Entity",
    from = "Column::BlobId",
    to = "super::active_storage_blobs::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  ActiveStorageBlobs,
}

impl Related<super::active_storage_blobs::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::ActiveStorageBlobs.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
