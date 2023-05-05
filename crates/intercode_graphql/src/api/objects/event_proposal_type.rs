use async_graphql::*;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use intercode_entities::{
  conventions, event_proposals, forms, model_ext::form_item_permissions::FormItemRole,
};
use intercode_policies::{
  policies::{EventProposalAction, EventProposalPolicy},
  AuthorizationInfo, FormResponsePolicy,
};
use seawater::loaders::ExpectModel;

use crate::{
  api::{interfaces::FormResponseImplementation, scalars::JsonScalar},
  load_one_by_model_id, loader_result_to_optional_single, loader_result_to_required_single,
  model_backed_type,
  policy_guard::PolicyGuard,
  QueryData,
};

use super::{
  active_storage_attachment_type::ActiveStorageAttachmentType, EventCategoryType, EventType,
  ModelBackedType, RegistrationPolicyType, UserConProfileType,
};
model_backed_type!(EventProposalType, event_proposals::Model);

impl EventProposalType {
  fn policy_guard(
    &self,
    action: EventProposalAction,
  ) -> PolicyGuard<
    '_,
    EventProposalPolicy,
    (conventions::Model, event_proposals::Model),
    event_proposals::Model,
  > {
    PolicyGuard::new(action, &self.model, move |model, ctx| {
      let model = model.clone();
      let ctx = ctx;
      let query_data = ctx.data::<QueryData>();

      Box::pin(async {
        let query_data = query_data?;
        let convention_loader = query_data.loaders().event_proposal_convention();
        let convention_result = convention_loader.load_one(model.id).await?;
        let convention = convention_result.expect_one()?;

        Ok((convention.clone(), model))
      })
    })
  }
}

#[Object(
  name = "EventProposal",
  guard = "self.policy_guard(EventProposalAction::Read)"
)]
impl EventProposalType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(
    name = "admin_notes",
    guard = "self.policy_guard(EventProposalAction::ReadAdminNotes)"
  )]
  async fn admin_notes(&self) -> Option<&str> {
    self.model.admin_notes.as_deref()
  }

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

  #[graphql(name = "event")]
  async fn event(&self, ctx: &Context<'_>) -> Result<Option<EventType>> {
    let loader_result = load_one_by_model_id!(event_proposal_event, ctx, self)?;
    Ok(loader_result_to_optional_single!(loader_result, EventType))
  }

  #[graphql(name = "event_category")]
  async fn event_category(&self, ctx: &Context<'_>) -> Result<EventCategoryType> {
    let loader_result = load_one_by_model_id!(event_proposal_event_category, ctx, self)?;
    Ok(loader_result_to_required_single!(
      loader_result,
      EventCategoryType
    ))
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

  async fn images(&self, ctx: &Context<'_>) -> Result<Vec<ActiveStorageAttachmentType>> {
    let blobs = ctx
      .data::<QueryData>()?
      .loaders()
      .event_proposal_attached_images
      .load_one(self.model.id)
      .await?
      .unwrap_or_default();

    Ok(
      blobs
        .into_iter()
        .map(ActiveStorageAttachmentType::new)
        .collect(),
    )
  }

  #[graphql(name = "length_seconds")]
  async fn length_seconds(&self) -> Option<i32> {
    self.model.length_seconds
  }

  async fn owner(&self, ctx: &Context<'_>) -> Result<UserConProfileType> {
    let loader_result = load_one_by_model_id!(event_proposal_owner, ctx, self)?;
    Ok(loader_result_to_required_single!(
      loader_result,
      UserConProfileType
    ))
  }

  #[graphql(name = "registration_policy")]
  async fn registration_policy(&self) -> RegistrationPolicyType {
    RegistrationPolicyType(
      self
        .model
        .registration_policy
        .as_ref()
        .map(|json| serde_json::from_value(json.clone()).unwrap_or_default())
        .unwrap_or_default(),
    )
  }

  async fn status(&self) -> Option<String> {
    self.model.status.as_ref().map(|status| {
      serde_json::to_value(status)
        .unwrap()
        .as_str()
        .unwrap()
        .to_string()
    })
  }

  #[graphql(name = "submitted_at")]
  async fn submitted_at(&self) -> Option<&NaiveDateTime> {
    self.model.submitted_at.as_ref()
  }

  async fn title(&self) -> Option<&str> {
    self.model.title.as_deref()
  }

  #[graphql(name = "updated_at")]
  async fn updated_at(&self) -> &NaiveDateTime {
    &self.model.updated_at
  }
}

#[async_trait]
impl FormResponseImplementation<event_proposals::Model> for EventProposalType {
  async fn get_form(&self, ctx: &Context<'_>) -> Result<forms::Model, Error> {
    let event_category = self.event_category(ctx).await?;
    let form_result = ctx
      .data::<QueryData>()?
      .loaders()
      .event_category_event_proposal_form()
      .load_one(event_category.get_model().id)
      .await?;
    Ok(form_result.expect_one()?.clone())
  }

  async fn get_team_member_name(&self, ctx: &Context<'_>) -> Result<String, Error> {
    let event_category = self.event_category(ctx).await?;
    Ok(event_category.get_model().team_member_name.to_string())
  }

  async fn current_user_form_item_viewer_role(
    &self,
    ctx: &Context<'_>,
  ) -> Result<FormItemRole, Error> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let convention_result = ctx
      .data::<QueryData>()?
      .loaders()
      .event_proposal_convention()
      .load_one(self.model.id)
      .await?;
    let convention = convention_result.expect_one()?;
    Ok(
      EventProposalPolicy::form_item_viewer_role(
        authorization_info,
        &(convention.clone(), self.model.clone()),
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
      .data::<QueryData>()?
      .loaders()
      .event_proposal_convention()
      .load_one(self.model.id)
      .await?;
    let convention = convention_result.expect_one()?;
    Ok(
      EventProposalPolicy::form_item_writer_role(
        authorization_info,
        &(convention.clone(), self.model.clone()),
      )
      .await,
    )
  }
}
