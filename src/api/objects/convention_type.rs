use crate::{
  cms_parent::{CmsParent, CmsParentTrait},
  conventions, pages, staff_positions, staff_positions_user_con_profiles, team_members,
  user_con_profiles, SchemaData,
};
use async_graphql::*;
use sea_orm::{ColumnTrait, Linked, ModelTrait, QueryFilter, QuerySelect, RelationTrait};

use super::{ModelBackedType, PageType, StaffPositionType, UserConProfileType};

use crate::model_backed_type;
model_backed_type!(ConventionType, conventions::Model);

pub struct ConventionToStaffPositions;

impl Linked for ConventionToStaffPositions {
  type FromEntity = conventions::Entity;
  type ToEntity = staff_positions::Entity;

  fn link(&self) -> Vec<sea_orm::LinkDef> {
    vec![staff_positions::Relation::Conventions.def().rev()]
  }
}

#[Object]
impl ConventionType {
  async fn id(&self) -> ID {
    ID(self.model.id.to_string())
  }

  async fn name(&self) -> &Option<String> {
    &self.model.name
  }

  #[graphql(name = "bio_eligible_user_con_profiles")]
  async fn bio_eligible_user_con_profiles(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<UserConProfileType>, Error> {
    let db = &ctx.data::<SchemaData>()?.db;

    let profiles: Vec<UserConProfileType> = self
      .model
      .find_related(user_con_profiles::Entity)
      .left_join(staff_positions_user_con_profiles::Entity)
      .left_join(team_members::Entity)
      .filter(
        staff_positions_user_con_profiles::Column::StaffPositionId
          .is_not_null()
          .or(team_members::Column::Id.is_not_null()),
      )
      .group_by(user_con_profiles::Column::Id)
      .all(db.as_ref())
      .await?
      .iter()
      .map(|model| UserConProfileType::new(model.to_owned()))
      .collect::<Vec<UserConProfileType>>();

    Ok(profiles)
  }

  async fn cms_page(
    &self,
    ctx: &Context<'_>,
    id: Option<ID>,
    slug: Option<String>,
    root_page: Option<bool>,
  ) -> Result<PageType, Error> {
    let db = &ctx.data::<SchemaData>()?.db;
    let cms_parent: CmsParent = self.model.clone().into();

    let scope = if let Some(id) = id {
      cms_parent
        .pages()
        .filter(pages::Column::Id.eq(id.parse::<i64>()?))
    } else if let Some(slug) = slug {
      cms_parent.pages().filter(pages::Column::Slug.eq(slug))
    } else if let Some(root_page) = root_page {
      if root_page {
        cms_parent.root_page()
      } else {
        return Err(Error::new("If rootPage is specified, it must be true"));
      }
    } else {
      return Err(Error::new("One of id, slug, or rootPage must be specified"));
    };

    scope
      .one(db.as_ref())
      .await?
      .ok_or_else(|| Error::new("Page not found"))
      .map(PageType::new)
  }

  #[graphql(name = "staff_position")]
  async fn staff_position(&self, ctx: &Context<'_>, id: ID) -> Result<StaffPositionType, Error> {
    let db = &ctx.data::<SchemaData>()?.db;

    self
      .model
      .find_linked(ConventionToStaffPositions)
      .filter(staff_positions::Column::Id.eq(id.parse::<u64>()?))
      .one(db.as_ref())
      .await?
      .ok_or_else(|| {
        Error::new(format!(
          "Staff position with ID {} not found in convention",
          id.as_str()
        ))
      })
      .map(StaffPositionType::new)
  }

  #[graphql(name = "user_con_profile")]
  async fn user_con_profile(&self, ctx: &Context<'_>, id: ID) -> Result<UserConProfileType, Error> {
    let db = &ctx.data::<SchemaData>()?.db;

    self
      .model
      .find_related(user_con_profiles::Entity)
      .filter(user_con_profiles::Column::Id.eq(id.parse::<u64>()?))
      .one(db.as_ref())
      .await?
      .ok_or(Error::new(format!(
        "No user con profile with ID {} in convention",
        id.as_str()
      )))
      .map(UserConProfileType::new)
  }
}
