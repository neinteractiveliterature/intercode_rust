use std::sync::Arc;

use async_graphql::*;
use async_trait::async_trait;
use intercode_entities::{event_proposals, forms, model_ext::form_item_permissions::FormItemRole};
use intercode_graphql_core::{
  load_one_by_id, load_one_by_model_id, model_backed_type, scalars::JsonScalar, ModelBackedType,
};
use intercode_graphql_loaders::LoaderManager;
use intercode_policies::{
  policies::{EventProposalAction, EventProposalPolicy},
  AuthorizationInfo, ModelBackedTypeGuardablePolicy,
};
use seawater::loaders::ExpectModel;

use crate::{
  form_response_implementation::FormResponseImplementation, policy_ext::FormResponsePolicy,
};

model_backed_type!(EventProposalFormsFields, event_proposals::Model);

#[Object(guard = "EventProposalPolicy::model_guard(EventProposalAction::Read, self)")]
impl EventProposalFormsFields {
  #[graphql(name = "current_user_form_item_viewer_role")]
  async fn current_user_form_item_viewer_role(
    &self,
    ctx: &Context<'_>,
  ) -> Result<FormItemRole, Error> {
    <Self as FormResponseImplementation<event_proposals::Model>>::current_user_form_item_viewer_role(
      self, ctx,
    ).await
  }

  #[graphql(name = "current_user_form_item_writer_role")]
  async fn current_user_form_item_writer_role(
    &self,
    ctx: &Context<'_>,
  ) -> Result<FormItemRole, Error> {
    <Self as FormResponseImplementation<event_proposals::Model>>::current_user_form_item_writer_role(
      self, ctx,
    ).await
  }

  #[graphql(name = "form_response_attrs_json")]
  async fn form_response_attrs_json(
    &self,
    ctx: &Context<'_>,
    item_identifiers: Option<Vec<String>>,
  ) -> Result<JsonScalar, Error> {
    <Self as FormResponseImplementation<event_proposals::Model>>::form_response_attrs_json(
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
    <Self as FormResponseImplementation<event_proposals::Model>>::form_response_attrs_json_with_rendered_markdown(
      self,
      ctx,
      item_identifiers,
    )
    .await
  }
}

#[async_trait]
impl FormResponseImplementation<event_proposals::Model> for EventProposalFormsFields {
  async fn get_form(&self, ctx: &Context<'_>) -> Result<forms::Model, Error> {
    let loader_result = load_one_by_model_id!(event_proposal_event_category, ctx, self)?;
    let event_category = loader_result.expect_one()?;

    let loader_result =
      load_one_by_id!(event_category_event_proposal_form, ctx, event_category.id)?;
    Ok(loader_result.expect_one()?.clone())
  }

  async fn get_team_member_name(&self, ctx: &Context<'_>) -> Result<String, Error> {
    let loader_result = load_one_by_model_id!(event_proposal_event_category, ctx, self)?;
    let event_category = loader_result.expect_one()?;
    Ok(event_category.team_member_name.to_string())
  }

  async fn current_user_form_item_viewer_role(
    &self,
    ctx: &Context<'_>,
  ) -> Result<FormItemRole, Error> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let convention_result = ctx
      .data::<Arc<LoaderManager>>()?
      .event_proposal_convention()
      .load_one(self.model.id)
      .await?;
    let convention = convention_result.expect_one()?;
    Ok(
      EventProposalPolicy::form_item_viewer_role(
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
      .event_proposal_convention()
      .load_one(self.model.id)
      .await?;
    let convention = convention_result.expect_one()?;
    Ok(
      EventProposalPolicy::form_item_writer_role(
        authorization_info,
        &(convention.clone(), self.get_model().clone()),
      )
      .await,
    )
  }
}
