use crate::api::policies::CmsContentPolicy;
use async_graphql::{Context, Error, Object};
use intercode_entities::{cms_content_model::CmsContentModel, pages, root_sites};
use intercode_graphql_core::query_data::QueryData;
use intercode_policies::{AuthorizationInfo, EntityPolicy, Policy, ReadManageAction};
use sea_orm::PaginatorTrait;
use std::borrow::Cow;

pub struct AbilityCmsFields<'a> {
  authorization_info: Cow<'a, AuthorizationInfo>,
}

impl<'a> AbilityCmsFields<'a> {
  pub fn new(authorization_info: Cow<'a, AuthorizationInfo>) -> Self {
    Self { authorization_info }
  }

  async fn can_perform_cms_content_action(
    &self,
    ctx: &Context<'_>,
    action: ReadManageAction,
  ) -> Result<bool, Error> {
    let convention = ctx.data::<QueryData>()?.convention();

    Ok(if let Some(convention) = convention {
      CmsContentPolicy::action_permitted(self.authorization_info.as_ref(), &action, convention)
        .await?
    } else {
      CmsContentPolicy::action_permitted(
        self.authorization_info.as_ref(),
        &action,
        &root_sites::Model {
          ..Default::default()
        },
      )
      .await?
    })
  }
}

#[Object]
impl<'a> AbilityCmsFields<'a> {
  #[graphql(name = "can_create_cms_files")]
  async fn can_create_cms_files(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    self
      .can_perform_cms_content_action(ctx, ReadManageAction::Manage)
      .await
  }

  #[graphql(name = "can_create_pages")]
  async fn can_create_pages(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    self
      .can_perform_cms_content_action(ctx, ReadManageAction::Manage)
      .await
  }

  #[graphql(name = "can_create_cms_partials")]
  async fn can_create_cms_partials(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    self
      .can_perform_cms_content_action(ctx, ReadManageAction::Manage)
      .await
  }

  #[graphql(name = "can_create_cms_layouts")]
  async fn can_create_cms_layouts(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    self
      .can_perform_cms_content_action(ctx, ReadManageAction::Manage)
      .await
  }

  #[graphql(name = "can_create_cms_navigation_items")]
  async fn can_create_cms_navigation_items(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    self
      .can_perform_cms_content_action(ctx, ReadManageAction::Manage)
      .await
  }

  #[graphql(name = "can_create_cms_variables")]
  async fn can_create_cms_variables(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    self
      .can_perform_cms_content_action(ctx, ReadManageAction::Manage)
      .await
  }

  #[graphql(name = "can_create_cms_graphql_queries")]
  async fn can_create_cms_graphql_queries(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    self
      .can_perform_cms_content_action(ctx, ReadManageAction::Manage)
      .await
  }

  #[graphql(name = "can_create_cms_content_groups")]
  async fn can_create_cms_content_groups(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    self
      .can_perform_cms_content_action(ctx, ReadManageAction::Manage)
      .await
  }

  #[graphql(name = "can_manage_any_cms_content")]
  async fn can_manage_any_cms_content(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    let query_data = ctx.data::<QueryData>()?;

    Ok(
      pages::Model::filter_by_parent(
        CmsContentPolicy::<pages::Model>::accessible_to(
          &self.authorization_info,
          &ReadManageAction::Manage,
        ),
        query_data.cms_parent(),
      )
      .count(query_data.db())
      .await?
        > 0,
    )
  }
}
