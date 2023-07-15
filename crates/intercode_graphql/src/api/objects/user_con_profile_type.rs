use std::borrow::Cow;
use std::sync::Arc;

use super::{
  AbilityType, ConventionType, SignupType, StaffPositionType, TeamMemberType, TicketType,
};
use crate::{api::interfaces::FormResponseImplementation, QueryData};
use async_graphql::*;
use async_trait::async_trait;
use intercode_entities::model_ext::form_item_permissions::FormItemRole;
use intercode_entities::{forms, user_con_profiles, UserNames};
use intercode_graphql_core::scalars::{DateScalar, JsonScalar};
use intercode_graphql_core::{
  load_one_by_model_id, loader_result_to_many, loader_result_to_optional_single, model_backed_type,
  ModelBackedType,
};
use intercode_graphql_loaders::LoaderManager;
use intercode_policies::policies::{UserConProfileAction, UserConProfilePolicy};
use intercode_policies::{AuthorizationInfo, FormResponsePolicy};
use intercode_store::partial_objects::UserConProfileStoreFields;
use pulldown_cmark::{html, Options, Parser};
use sea_orm::EntityTrait;
use seawater::loaders::ExpectModel;

model_backed_type!(UserConProfileApiFields, user_con_profiles::Model);

#[Object(name = "UserConProfile")]
impl UserConProfileApiFields {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn ability(&self, ctx: &Context<'_>) -> Result<AbilityType> {
    let query_data = ctx.data::<QueryData>()?;
    let user = load_one_by_model_id!(user_con_profile_user, ctx, self)?;
    let authorization_info =
      AuthorizationInfo::new(query_data.db().clone(), user.try_one().cloned(), None, None);

    Ok(AbilityType::new(Cow::Owned(authorization_info)))
  }

  #[graphql(name = "accepted_clickwrap_agreement")]
  async fn accepted_clickwrap_agreement(&self) -> bool {
    self.model.accepted_clickwrap_agreement
  }

  async fn address(&self) -> Option<&str> {
    self.model.address.as_deref()
  }

  #[graphql(name = "bio_html")]
  async fn bio_html(&self) -> Option<String> {
    if let Some(bio) = &self.model.bio {
      let mut options = Options::empty();
      options.insert(Options::ENABLE_STRIKETHROUGH);
      options.insert(Options::ENABLE_FOOTNOTES);
      options.insert(Options::ENABLE_SMART_PUNCTUATION);
      options.insert(Options::ENABLE_TABLES);
      let parser = Parser::new_ext(bio, options);

      let mut html_output = String::new();
      html::push_html(&mut html_output, parser);
      Some(html_output)
    } else {
      None
    }
  }

  #[graphql(name = "bio_name")]
  async fn bio_name(&self) -> String {
    self.model.bio_name()
  }

  #[graphql(name = "birth_date")]
  async fn birth_date(&self) -> Result<Option<DateScalar>> {
    self.model.birth_date.map(DateScalar::try_from).transpose()
  }

  async fn city(&self) -> Option<&str> {
    self.model.city.as_deref()
  }

  async fn convention(&self, ctx: &Context<'_>) -> Result<ConventionType, Error> {
    let loader = &ctx.data::<Arc<LoaderManager>>()?.conventions_by_id();
    let loader_result = loader.load_one(self.model.convention_id).await?;
    Ok(ConventionType::new(loader_result.expect_one()?.clone()))
  }

  async fn country(&self) -> Option<&str> {
    self.model.country.as_deref()
  }

  async fn email(&self, ctx: &Context<'_>) -> Result<String, Error> {
    let loader = ctx.data::<Arc<LoaderManager>>()?.user_con_profile_user();

    Ok(
      loader
        .load_one(self.model.id)
        .await?
        .expect_one()?
        .email
        .to_owned(),
    )
  }

  #[graphql(name = "first_name")]
  async fn first_name(&self) -> &str {
    self.model.first_name.as_str()
  }

  #[graphql(name = "gravatar_enabled")]
  async fn gravatar_enabled(&self) -> bool {
    self.model.gravatar_enabled
  }

  #[graphql(name = "gravatar_url")]
  async fn gravatar_url(&self, ctx: &Context<'_>) -> Result<String, Error> {
    if self.model.gravatar_enabled {
      let loader = &ctx.data::<Arc<LoaderManager>>()?.users_by_id();
      let loader_result = loader.load_one(self.model.user_id).await?;
      let model = loader_result.expect_one()?;
      Ok(format!(
        "https://gravatar.com/avatar/{:x}",
        md5::compute(model.email.trim().to_lowercase())
      ))
    } else {
      Ok(format!(
        "https://gravatar.com/avatar/{:x}",
        md5::compute("badrequest")
      ))
    }
  }

  #[graphql(
    name = "ical_secret",
    guard = "self.simple_policy_guard::<UserConProfilePolicy>(UserConProfileAction::ReadPersonalInfo)"
  )]
  async fn ical_secret(&self) -> &str {
    &self.model.ical_secret
  }

  #[graphql(name = "last_name")]
  async fn last_name(&self) -> &str {
    self.model.last_name.as_str()
  }

  #[graphql(name = "mobile_phone")]
  async fn mobile_phone(&self) -> Option<&str> {
    self.model.mobile_phone.as_deref()
  }

  async fn name(&self) -> String {
    self.model.name()
  }

  #[graphql(name = "name_inverted")]
  async fn name_inverted(&self) -> String {
    self.model.name_inverted()
  }

  #[graphql(name = "name_without_nickname")]
  async fn name_without_nickname(&self) -> String {
    self.model.name_without_nickname()
  }

  async fn nickname(&self) -> Option<&str> {
    self.model.nickname.as_deref()
  }

  async fn signups(&self, ctx: &Context<'_>) -> Result<Vec<SignupType>> {
    let signups_result = load_one_by_model_id!(user_con_profile_signups, ctx, self)?;
    Ok(loader_result_to_many!(signups_result, SignupType))
  }

  #[graphql(name = "site_admin")]
  async fn site_admin(&self, ctx: &Context<'_>) -> Result<bool> {
    let user = ctx
      .data::<Arc<LoaderManager>>()?
      .user_con_profile_user()
      .load_one(self.model.id)
      .await?;

    Ok(user.expect_one()?.site_admin.unwrap_or(false))
  }

  #[graphql(name = "staff_positions")]
  async fn staff_positions(&self, ctx: &Context<'_>) -> Result<Vec<StaffPositionType>, Error> {
    let loader_result = load_one_by_model_id!(user_con_profile_staff_positions, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, StaffPositionType))
  }

  async fn state(&self) -> Option<&str> {
    self.model.state.as_deref()
  }

  #[graphql(name = "team_members")]
  async fn team_members(&self, ctx: &Context<'_>) -> Result<Vec<TeamMemberType>, Error> {
    let loader_result = load_one_by_model_id!(user_con_profile_team_members, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, TeamMemberType))
  }

  async fn ticket(&self, ctx: &Context<'_>) -> Result<Option<TicketType>, Error> {
    let loader_result = load_one_by_model_id!(user_con_profile_ticket, ctx, self)?;
    Ok(loader_result_to_optional_single!(loader_result, TicketType))
  }

  #[graphql(name = "user_id")]
  async fn user_id(&self) -> ID {
    self.model.user_id.into()
  }

  async fn zipcode(&self) -> Option<&str> {
    self.model.zipcode.as_deref()
  }

  // STUFF FOR FORM_RESPONSE_INTERFACE

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
impl FormResponseImplementation<user_con_profiles::Model> for UserConProfileApiFields {
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

#[derive(MergedObject)]
#[graphql(name = "UserConProfileType")]
pub struct UserConProfileType(UserConProfileApiFields, UserConProfileStoreFields);

impl ModelBackedType for UserConProfileType {
  type Model = user_con_profiles::Model;

  fn new(model: Self::Model) -> Self {
    Self(
      UserConProfileApiFields::new(model.clone()),
      UserConProfileStoreFields::new(model),
    )
  }

  fn get_model(&self) -> &Self::Model {
    self.0.get_model()
  }

  fn into_model(self) -> Self::Model {
    self.0.into_model()
  }
}
