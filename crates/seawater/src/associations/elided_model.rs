use std::{convert::Infallible, str::FromStr};

use sea_orm::{
  sea_query, ColumnTrait, EntityName, EntityTrait, EnumIter, FromQueryResult, Iden, IdenStatic,
  ModelTrait, PrimaryKeyToColumn, PrimaryKeyTrait, RelationTrait,
};

#[derive(Iden, Copy, Clone, Debug, EnumIter)]
enum ElidedColumn {
  Elided,
}

impl FromStr for ElidedColumn {
  type Err = Infallible;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(ElidedColumn::Elided)
  }
}

#[derive(Iden, Copy, Clone, EnumIter, Debug)]
enum ElidedEntityName {
  Elided,
}

impl Default for ElidedEntityName {
  fn default() -> Self {
    ElidedEntityName::Elided
  }
}

impl IdenStatic for ElidedEntityName {
  fn as_str(&self) -> &str {
    "Elided"
  }
}

impl EntityName for ElidedEntityName {
  fn table_name(&self) -> &str {
    "elided"
  }
}

impl ColumnTrait for ElidedColumn {
  type EntityName = ElidedEntityName;

  fn def(&self) -> sea_orm::ColumnDef {
    todo!()
  }
}

impl IdenStatic for ElidedColumn {
  fn as_str(&self) -> &str {
    todo!()
  }
}

#[derive(Debug, EnumIter)]
enum ElidedRelation {
  Elided,
}

impl RelationTrait for ElidedRelation {
  fn def(&self) -> sea_orm::RelationDef {
    todo!()
  }
}

#[derive(Debug, EnumIter, Iden, Copy, Clone)]
enum ElidedPrimaryKey {
  Elided,
}

impl IdenStatic for ElidedPrimaryKey {
  fn as_str(&self) -> &str {
    todo!()
  }
}

impl PrimaryKeyTrait for ElidedPrimaryKey {
  type ValueType = i64;

  fn auto_increment() -> bool {
    todo!()
  }
}

impl PrimaryKeyToColumn for ElidedPrimaryKey {
  type Column = ElidedColumn;

  fn into_column(self) -> Self::Column {
    ElidedColumn::Elided
  }

  fn from_column(col: Self::Column) -> Option<Self>
  where
    Self: Sized,
  {
    Some(Self::Elided)
  }
}

#[derive(Debug, Clone, Default, Copy, Iden)]
#[iden = "elided_entity"]
struct ElidedEntity;

impl IdenStatic for ElidedEntity {
  fn as_str(&self) -> &str {
    "elided_entity"
  }
}

impl EntityName for ElidedEntity {
  fn table_name(&self) -> &str {
    todo!()
  }
}

impl EntityTrait for ElidedEntity {
  type Model = ElidedModel;
  type Column = ElidedColumn;
  type Relation = ElidedRelation;
  type PrimaryKey = ElidedPrimaryKey;
}

#[derive(Debug, Clone)]
pub struct ElidedModel;

impl ModelTrait for ElidedModel {
  type Entity = ElidedEntity;

  fn get(&self, c: <Self::Entity as EntityTrait>::Column) -> sea_orm::Value {
    todo!()
  }

  fn set(&mut self, c: <Self::Entity as EntityTrait>::Column, v: sea_orm::Value) {
    todo!()
  }
}

impl FromQueryResult for ElidedModel {
  fn from_query_result(res: &sea_orm::QueryResult, pre: &str) -> Result<Self, sea_orm::DbErr> {
    todo!()
  }
}
