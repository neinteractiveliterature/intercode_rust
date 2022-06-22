use crate::{
  api::enums::{SignupMode, SiteMode, TicketMode, TimezoneMode},
  QueryData, SchemaData,
};
use async_graphql::*;
use chrono::{DateTime, Utc};
use intercode_entities::{
  cms_parent::{CmsParent, CmsParentTrait},
  conventions, pages, staff_positions, staff_positions_user_con_profiles, team_members,
  user_con_profiles,
};
use sea_orm::{
  ColumnTrait, EntityTrait, Linked, ModelTrait, QueryFilter, QuerySelect, RelationTrait,
};

use super::{
  CmsLayoutType, CmsNavigationItemType, ModelBackedType, PageType, StaffPositionType,
  UserConProfileType,
};

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
  pub async fn id(&self) -> ID {
    ID(self.model.id.to_string())
  }

  async fn name(&self) -> &Option<String> {
    &self.model.name
  }

  #[graphql(name = "accepting_proposals")]
  async fn accepting_proposals(&self) -> bool {
    self.model.accepting_proposals.unwrap_or(false)
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

  async fn canceled(&self) -> bool {
    self.model.canceled
  }

  #[graphql(name = "clickwrap_agreement")]
  async fn clickwrap_agreement(&self) -> Option<&str> {
    self.model.clickwrap_agreement.as_deref()
  }

  #[graphql(name = "cms_navigation_items")]
  pub async fn cms_navigation_items(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<CmsNavigationItemType>, Error> {
    let schema_data = ctx.data::<SchemaData>()?;

    Ok(
      self
        .model
        .cms_navigation_items()
        .all(schema_data.db.as_ref())
        .await?
        .iter()
        .map(|item| CmsNavigationItemType::new(item.to_owned()))
        .collect(),
    )
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

  #[graphql(name = "default_layout")]
  pub async fn default_layout(&self, ctx: &Context<'_>) -> Result<CmsLayoutType, Error> {
    let schema_data = ctx.data::<SchemaData>()?;

    self
      .model
      .default_layout()
      .one(schema_data.db.as_ref())
      .await?
      .ok_or_else(|| {
        Error::new(format!(
          "Default layout not found for {}",
          self
            .model
            .name
            .as_ref()
            .map(|name| name.as_str())
            .unwrap_or("convention")
        ))
      })
      .map(CmsLayoutType::new)
  }

  async fn domain(&self) -> &str {
    self.model.domain.as_str()
  }

  #[graphql(name = "ends_at")]
  async fn ends_at(&self) -> Option<DateTime<Utc>> {
    self
      .model
      .ends_at
      .map(|t| DateTime::<Utc>::from_utc(t, Utc))
  }

  async fn language(&self) -> &str {
    self.model.language.as_str()
  }

  #[graphql(name = "my_profile")]
  async fn my_profile(&self, ctx: &Context<'_>) -> Result<Option<UserConProfileType>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    let convention_id = query_data.convention.as_ref().as_ref().map(|c| c.id);

    if convention_id == Some(self.model.id) {
      Ok(
        query_data
          .user_con_profile
          .as_ref()
          .as_ref()
          .map(|ucp| UserConProfileType::new(ucp.to_owned())),
      )
    } else if let Some(user) = query_data.current_user.as_ref() {
      let schema_data = ctx.data::<SchemaData>()?;

      user_con_profiles::Entity::find()
        .filter(
          user_con_profiles::Column::ConventionId
            .eq(self.model.id)
            .and(user_con_profiles::Column::UserId.eq(user.id)),
        )
        .one(schema_data.db.as_ref())
        .await
        .map(|result| result.map(|ucp| UserConProfileType::new(ucp)))
        .map_err(|e| async_graphql::Error::new(e.to_string()))
    } else {
      Ok(None)
    }
  }

  #[graphql(name = "signup_mode")]
  async fn signup_mode(&self) -> Result<SignupMode, Error> {
    self.model.signup_mode.as_str().try_into()
  }

  #[graphql(name = "site_mode")]
  async fn site_mode(&self) -> Result<SiteMode, Error> {
    self.model.site_mode.as_str().try_into()
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

  #[graphql(name = "starts_at")]
  async fn starts_at(&self) -> Option<DateTime<Utc>> {
    self
      .model
      .starts_at
      .map(|t| DateTime::<Utc>::from_utc(t, Utc))
  }

  #[graphql(name = "stripe_account_id")]
  async fn stripe_account_id(&self) -> Option<&str> {
    self.model.stripe_account_id.as_deref()
  }

  #[graphql(name = "stripe_publishable_key")]
  async fn stripe_publishable_key(&self) -> Option<String> {
    std::env::var("STRIPE_PUBLISHABLE_KEY").ok()
  }

  #[graphql(name = "ticket_mode")]
  async fn ticket_mode(&self) -> Result<TicketMode, Error> {
    self.model.ticket_mode.as_str().try_into()
  }

  #[graphql(name = "ticket_name")]
  async fn ticket_name(&self) -> &str {
    self.model.ticket_name.as_str()
  }

  async fn ticket_name_plural(&self) -> String {
    intercode_inflector::inflector::Inflector::to_plural(self.model.ticket_name.as_str())
  }

  #[graphql(name = "tickets_available_for_purchase")]
  async fn tickets_available_for_purchase(&self) -> bool {
    self.model.tickets_available_for_purchase()
  }

  #[graphql(name = "timezone_mode")]
  async fn timezone_mode(&self) -> Result<TimezoneMode, Error> {
    self.model.timezone_mode.as_str().try_into()
  }

  #[graphql(name = "timezone_name")]
  async fn timezone_name(&self) -> Option<&str> {
    self.model.timezone_name.as_deref()
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
      .ok_or_else(|| {
        Error::new(format!(
          "No user con profile with ID {} in convention",
          id.as_str()
        ))
      })
      .map(UserConProfileType::new)
  }
}
