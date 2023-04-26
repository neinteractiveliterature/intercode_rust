use sea_orm::{
  sea_query::SelectStatement, ColumnTrait, EntityTrait, ModelTrait, QueryFilter, Select,
};

use crate::{
  cms_content_group_associations, cms_content_groups, cms_files, cms_graphql_queries, cms_layouts,
  cms_parent::CmsParent, cms_partials, cms_variables, pages,
};

fn cms_content_groups_scope_for_convention_id(
  convention_id: Option<i64>,
) -> Select<cms_content_groups::Entity> {
  if let Some(convention_id) = convention_id {
    cms_content_groups::Entity::find()
      .filter(cms_content_groups::Column::ParentId.eq(convention_id))
      .filter(cms_content_groups::Column::ParentType.eq("Convention"))
  } else {
    cms_content_groups::Entity::find()
      .filter(cms_content_groups::Column::ParentType.is_null())
      .filter(cms_content_groups::Column::ParentId.is_null())
  }
}

pub trait CmsContentModel: sea_orm::ModelTrait + Sync {
  fn convention_id(&self) -> Option<i64>;
  fn cms_content_group_association_model_name() -> &'static str;
  fn id_column() -> <<Self as ModelTrait>::Entity as EntityTrait>::Column;
  fn parent_id_column() -> <<Self as ModelTrait>::Entity as EntityTrait>::Column;
  fn parent_type_column() -> <<Self as ModelTrait>::Entity as EntityTrait>::Column;

  fn cms_content_groups_scope(&self) -> Select<cms_content_groups::Entity> {
    let scope = cms_content_groups_scope_for_convention_id(self.convention_id());

    scope
      .inner_join(cms_content_group_associations::Entity)
      .filter(
        cms_content_group_associations::Column::ContentType
          .eq(Self::cms_content_group_association_model_name()),
      )
      .filter(cms_content_group_associations::Column::ContentId.eq(self.get(Self::id_column())))
  }

  fn filter_by_id_in(
    scope: Select<<Self as ModelTrait>::Entity>,
    subquery: SelectStatement,
  ) -> Select<<Self as ModelTrait>::Entity> {
    scope.filter(Self::id_column().in_subquery(subquery))
  }

  fn filter_by_parent_id(
    scope: Select<<Self as ModelTrait>::Entity>,
    parent_type: &str,
    subquery: SelectStatement,
  ) -> Select<<Self as ModelTrait>::Entity> {
    scope
      .filter(Self::parent_type_column().eq(parent_type))
      .filter(Self::parent_id_column().in_subquery(subquery))
  }

  fn filter_by_parent(
    scope: Select<<Self as ModelTrait>::Entity>,
    parent: &CmsParent,
  ) -> Select<<Self as ModelTrait>::Entity> {
    match parent {
      CmsParent::Convention(convention) => scope
        .filter(Self::parent_type_column().eq("Convention"))
        .filter(Self::parent_id_column().eq(convention.id)),
      CmsParent::RootSite(root_site) => scope
        .filter(Self::parent_type_column().eq("RootSite"))
        .filter(Self::parent_id_column().eq(root_site.id)),
    }
  }
}

impl CmsContentModel for cms_content_groups::Model {
  fn convention_id(&self) -> Option<i64> {
    if !matches!(self.parent_type.as_deref(), Some("Convention")) {
      return None;
    }

    self.parent_id
  }

  fn cms_content_group_association_model_name() -> &'static str {
    "CmsContentGroup"
  }

  fn id_column() -> <<Self as ModelTrait>::Entity as EntityTrait>::Column {
    cms_content_groups::Column::Id
  }

  fn parent_id_column() -> <<Self as ModelTrait>::Entity as EntityTrait>::Column {
    cms_content_groups::Column::ParentId
  }

  fn parent_type_column() -> <<Self as ModelTrait>::Entity as EntityTrait>::Column {
    cms_content_groups::Column::ParentType
  }
}

impl CmsContentModel for cms_files::Model {
  fn convention_id(&self) -> Option<i64> {
    if !matches!(self.parent_type.as_deref(), Some("Convention")) {
      return None;
    }

    self.parent_id
  }

  fn cms_content_group_association_model_name() -> &'static str {
    "CmsFile"
  }

  fn id_column() -> <<Self as ModelTrait>::Entity as EntityTrait>::Column {
    cms_files::Column::Id
  }

  fn parent_id_column() -> <<Self as ModelTrait>::Entity as EntityTrait>::Column {
    cms_files::Column::ParentId
  }

  fn parent_type_column() -> <<Self as ModelTrait>::Entity as EntityTrait>::Column {
    cms_files::Column::ParentType
  }
}

impl CmsContentModel for cms_graphql_queries::Model {
  fn convention_id(&self) -> Option<i64> {
    if !matches!(self.parent_type.as_deref(), Some("Convention")) {
      return None;
    }

    self.parent_id
  }

  fn cms_content_group_association_model_name() -> &'static str {
    "CmsGraphqlQuery"
  }

  fn id_column() -> <<Self as ModelTrait>::Entity as EntityTrait>::Column {
    cms_graphql_queries::Column::Id
  }

  fn parent_id_column() -> <<Self as ModelTrait>::Entity as EntityTrait>::Column {
    cms_graphql_queries::Column::ParentId
  }

  fn parent_type_column() -> <<Self as ModelTrait>::Entity as EntityTrait>::Column {
    cms_graphql_queries::Column::ParentType
  }
}

impl CmsContentModel for cms_layouts::Model {
  fn convention_id(&self) -> Option<i64> {
    if !matches!(self.parent_type.as_deref(), Some("Convention")) {
      return None;
    }

    self.parent_id
  }

  fn cms_content_group_association_model_name() -> &'static str {
    "CmsLayout"
  }

  fn id_column() -> <<Self as ModelTrait>::Entity as EntityTrait>::Column {
    cms_layouts::Column::Id
  }

  fn parent_id_column() -> <<Self as ModelTrait>::Entity as EntityTrait>::Column {
    cms_layouts::Column::ParentId
  }

  fn parent_type_column() -> <<Self as ModelTrait>::Entity as EntityTrait>::Column {
    cms_layouts::Column::ParentType
  }
}

impl CmsContentModel for cms_partials::Model {
  fn convention_id(&self) -> Option<i64> {
    if !matches!(self.parent_type.as_deref(), Some("Convention")) {
      return None;
    }

    self.parent_id
  }

  fn cms_content_group_association_model_name() -> &'static str {
    "CmsPartial"
  }

  fn id_column() -> <<Self as ModelTrait>::Entity as EntityTrait>::Column {
    cms_partials::Column::Id
  }

  fn parent_id_column() -> <<Self as ModelTrait>::Entity as EntityTrait>::Column {
    cms_partials::Column::ParentId
  }

  fn parent_type_column() -> <<Self as ModelTrait>::Entity as EntityTrait>::Column {
    cms_partials::Column::ParentType
  }
}

impl CmsContentModel for cms_variables::Model {
  fn convention_id(&self) -> Option<i64> {
    if !matches!(self.parent_type.as_deref(), Some("Convention")) {
      return None;
    }

    self.parent_id
  }

  fn cms_content_group_association_model_name() -> &'static str {
    "CmsVariable"
  }

  fn id_column() -> <<Self as ModelTrait>::Entity as EntityTrait>::Column {
    cms_variables::Column::Id
  }

  fn parent_id_column() -> <<Self as ModelTrait>::Entity as EntityTrait>::Column {
    cms_variables::Column::ParentId
  }

  fn parent_type_column() -> <<Self as ModelTrait>::Entity as EntityTrait>::Column {
    cms_variables::Column::ParentType
  }
}

impl CmsContentModel for pages::Model {
  fn convention_id(&self) -> Option<i64> {
    if !matches!(self.parent_type.as_deref(), Some("Convention")) {
      return None;
    }

    self.parent_id
  }

  fn cms_content_group_association_model_name() -> &'static str {
    "Page"
  }

  fn id_column() -> <<Self as ModelTrait>::Entity as EntityTrait>::Column {
    pages::Column::Id
  }

  fn parent_id_column() -> <<Self as ModelTrait>::Entity as EntityTrait>::Column {
    pages::Column::ParentId
  }

  fn parent_type_column() -> <<Self as ModelTrait>::Entity as EntityTrait>::Column {
    pages::Column::ParentType
  }
}
