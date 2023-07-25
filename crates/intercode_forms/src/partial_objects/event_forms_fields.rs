use std::sync::Arc;

use async_graphql::*;
use async_trait::async_trait;
use intercode_entities::{events, forms, model_ext::form_item_permissions::FormItemRole};
use intercode_graphql_core::{model_backed_type, scalars::JsonScalar, ModelBackedType};
use intercode_graphql_loaders::LoaderManager;
use intercode_policies::{
  policies::{EventAction, EventPolicy},
  AuthorizationInfo, ModelBackedTypeGuardablePolicy,
};
use seawater::loaders::ExpectModel;

use crate::{
  form_response_implementation::FormResponseImplementation, policy_ext::FormResponsePolicy,
};

model_backed_type!(EventFormsFields, events::Model);

#[Object(guard = "EventPolicy::model_guard(EventAction::Read, self)")]
impl EventFormsFields {
  #[graphql(name = "current_user_form_item_viewer_role")]
  async fn form_item_viewer_role(&self, ctx: &Context<'_>) -> Result<FormItemRole> {
    <Self as FormResponseImplementation<events::Model>>::current_user_form_item_viewer_role(
      self, ctx,
    )
    .await
  }

  #[graphql(name = "current_user_form_item_writer_role")]
  async fn form_item_writer_role(&self, ctx: &Context<'_>) -> Result<FormItemRole> {
    <Self as FormResponseImplementation<events::Model>>::current_user_form_item_writer_role(
      self, ctx,
    )
    .await
  }

  #[graphql(name = "form_response_attrs_json")]
  async fn form_response_attrs_json(
    &self,
    ctx: &Context<'_>,
    item_identifiers: Option<Vec<String>>,
  ) -> Result<JsonScalar, Error> {
    <Self as FormResponseImplementation<events::Model>>::form_response_attrs_json(
      self,
      ctx,
      item_identifiers,
    )
    .await
  }

  #[graphql(name = "form_response_attrs_json_with_rendered_markdown")]
  async fn form_response_attrs_json_with_rendered_markdown(
    &self,
    ctx: &Context<'_>,
    item_identifiers: Option<Vec<String>>,
  ) -> Result<JsonScalar, Error> {
    <Self as FormResponseImplementation<events::Model>>::form_response_attrs_json_with_rendered_markdown(
      self,
      ctx,
      item_identifiers,
    )
    .await
  }
}

#[async_trait]
impl FormResponseImplementation<events::Model> for EventFormsFields {
  async fn get_form(&self, ctx: &Context<'_>) -> Result<forms::Model, Error> {
    let loaders = ctx.data::<Arc<LoaderManager>>()?;
    let event_category_result = loaders
      .event_event_category()
      .load_one(self.model.id)
      .await?;
    let event_category = event_category_result.expect_one()?;

    Ok(
      loaders
        .event_category_event_form()
        .load_one(event_category.id)
        .await?
        .expect_one()?
        .clone(),
    )
  }

  async fn get_team_member_name(&self, ctx: &Context<'_>) -> Result<String, Error> {
    let loaders = ctx.data::<Arc<LoaderManager>>()?;
    let event_category_result = loaders
      .event_event_category()
      .load_one(self.model.id)
      .await?;
    let event_category = event_category_result.expect_one()?;

    Ok(event_category.team_member_name.clone())
  }

  async fn current_user_form_item_viewer_role(
    &self,
    ctx: &Context<'_>,
  ) -> Result<FormItemRole, Error> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let convention_result = ctx
      .data::<Arc<LoaderManager>>()?
      .event_convention()
      .load_one(self.model.id)
      .await?;
    let convention = convention_result.expect_one()?;
    Ok(
      EventPolicy::form_item_viewer_role(
        authorization_info,
        &(convention.clone(), self.get_model().clone()),
      )
      .await,
    )
  }

  async fn current_user_form_item_writer_role(
    &self,
    ctx: &Context<'_>,
  ) -> Result<FormItemRole, Error> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let convention_result = ctx
      .data::<Arc<LoaderManager>>()?
      .event_convention()
      .load_one(self.model.id)
      .await?;
    let convention = convention_result.expect_one()?;
    Ok(
      EventPolicy::form_item_writer_role(
        authorization_info,
        &(convention.clone(), self.get_model().clone()),
      )
      .await,
    )
  }
}
