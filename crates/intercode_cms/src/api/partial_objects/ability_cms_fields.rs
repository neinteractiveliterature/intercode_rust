use crate::api::policies::{CmsContentPolicy, NotificationTemplatePolicy};
use async_graphql::{Context, Error, Object};
use intercode_entities::{
  cms_content_model::CmsContentModel, notification_templates, pages, root_sites,
};
use intercode_graphql_core::query_data::QueryData;
use intercode_policies::{AuthorizationInfo, EntityPolicy, Policy, ReadManageAction};
use sea_orm::PaginatorTrait;
use std::sync::Arc;

pub struct AbilityCmsFields {
  authorization_info: Arc<AuthorizationInfo>,
}

impl AbilityCmsFields {
  pub fn new(authorization_info: Arc<AuthorizationInfo>) -> Self {
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
impl AbilityCmsFields {
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

  #[graphql(name = "can_update_notification_templates")]
  async fn can_update_notification_templates(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    let Some(convention) = ctx.data::<QueryData>()?.convention() else {
      return Ok(false);
    };

    Ok(
      NotificationTemplatePolicy::action_permitted(
        self.authorization_info.as_ref(),
        &ReadManageAction::Manage,
        &notification_templates::Model {
          convention_id: convention.id,
          ..Default::default()
        },
      )
      .await?,
    )
  }
}
