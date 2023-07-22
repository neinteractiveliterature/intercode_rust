use std::sync::Arc;

use async_graphql::*;
use async_trait::async_trait;
use intercode_entities::{
  forms, model_ext::form_item_permissions::FormItemRole, user_con_profiles,
};
use intercode_graphql_core::{model_backed_type, scalars::JsonScalar};
use intercode_graphql_loaders::LoaderManager;
use intercode_policies::{policies::UserConProfilePolicy, AuthorizationInfo};
use seawater::loaders::ExpectModel;

use crate::{
  form_response_implementation::FormResponseImplementation, policy_ext::FormResponsePolicy,
};

model_backed_type!(UserConProfileFormsFields, user_con_profiles::Model);

#[Object]
impl UserConProfileFormsFields {
  #[graphql(name = "current_user_form_item_viewer_role")]
  async fn form_item_viewer_role(&self, ctx: &Context<'_>) -> Result<FormItemRole> {
    <Self as FormResponseImplementation<user_con_profiles::Model>>::current_user_form_item_viewer_role(
      self, ctx,
    )
    .await
  }

  #[graphql(name = "current_user_form_item_writer_role")]
  async fn form_item_writer_role(&self, ctx: &Context<'_>) -> Result<FormItemRole> {
    <Self as FormResponseImplementation<user_con_profiles::Model>>::current_user_form_item_writer_role(
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
    <Self as FormResponseImplementation<user_con_profiles::Model>>::form_response_attrs_json(
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
    <Self as FormResponseImplementation<user_con_profiles::Model>>::form_response_attrs_json_with_rendered_markdown(
      self,
      ctx,
      item_identifiers,
    )
    .await
  }
}

#[async_trait]
impl FormResponseImplementation<user_con_profiles::Model> for UserConProfileFormsFields {
  async fn get_form(&self, ctx: &Context<'_>) -> Result<forms::Model, Error> {
    let loaders = ctx.data::<Arc<LoaderManager>>()?;
    loaders
      .convention_user_con_profile_form()
      .load_one(self.model.convention_id)
      .await?
      .expect_one()
      .cloned()
  }

  async fn get_team_member_name(&self, _ctx: &Context<'_>) -> Result<String, Error> {
    Ok("team member".to_string())
  }

  async fn current_user_form_item_viewer_role(
    &self,
    ctx: &Context<'_>,
  ) -> Result<FormItemRole, Error> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    Ok(UserConProfilePolicy::form_item_viewer_role(authorization_info, &self.model).await)
  }

  async fn current_user_form_item_writer_role(
    &self,
    ctx: &Context<'_>,
  ) -> Result<FormItemRole, Error> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    Ok(UserConProfilePolicy::form_item_writer_role(authorization_info, &self.model).await)
  }
}
