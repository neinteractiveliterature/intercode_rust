use std::sync::Arc;

use async_graphql::*;
use axum::async_trait;
use intercode_entities::{
  conventions, signup_requests, signups, staff_positions, team_members, tickets, user_con_profiles,
  users, UserNames,
};
use intercode_graphql_core::query_data::QueryData;
use intercode_graphql_core::scalars::DateScalar;
use intercode_graphql_core::{
  load_one_by_model_id, loader_result_to_many, loader_result_to_optional_single,
  loader_result_to_required_single, model_backed_type, ModelBackedType,
};
use intercode_graphql_loaders::LoaderManager;
use intercode_policies::policies::{UserConProfileAction, UserConProfilePolicy};
use intercode_policies::{AuthorizationInfo, ModelBackedTypeGuardablePolicy};
use pulldown_cmark::{html, Options, Parser};
use seawater::loaders::ExpectModel;

use super::ability_users_fields::AbilityUsersFields;

model_backed_type!(UserConProfileUsersFields, user_con_profiles::Model);

#[async_trait]
pub trait UserConProfileUsersExtensions
where
  Self: ModelBackedType<Model = user_con_profiles::Model>,
{
  async fn ability(&self, ctx: &Context<'_>) -> Result<AbilityUsersFields> {
    let query_data = ctx.data::<QueryData>()?;
    let user = load_one_by_model_id!(user_con_profile_user, ctx, self)?;
    let authorization_info =
      AuthorizationInfo::new(query_data.db().clone(), user.try_one().cloned(), None, None);

    Ok(AbilityUsersFields::new(Arc::new(authorization_info)))
  }

  async fn convention<T: ModelBackedType<Model = conventions::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<T, Error> {
    let loader_result = load_one_by_model_id!(user_con_profile_convention, ctx, self)?;
    Ok(loader_result_to_required_single!(loader_result, T))
  }

  async fn signups<T: ModelBackedType<Model = signups::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<T>> {
    let loader_result = load_one_by_model_id!(user_con_profile_signups, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, T))
  }

  async fn signup_requests<T: ModelBackedType<Model = signup_requests::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<T>> {
    let loader_result = load_one_by_model_id!(user_con_profile_signup_requests, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, T))
  }

  async fn staff_positions<T: ModelBackedType<Model = staff_positions::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<T>, Error> {
    let loader_result = load_one_by_model_id!(user_con_profile_staff_positions, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, T))
  }

  async fn team_members<T: ModelBackedType<Model = team_members::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<T>, Error> {
    let loader_result = load_one_by_model_id!(user_con_profile_team_members, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, T))
  }

  async fn ticket<T: ModelBackedType<Model = tickets::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<T>, Error> {
    let loader_result = load_one_by_model_id!(user_con_profile_ticket, ctx, self)?;
    Ok(loader_result_to_optional_single!(loader_result, T))
  }

  async fn user<T: ModelBackedType<Model = users::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<T, Error> {
    let loader_result = load_one_by_model_id!(user_con_profile_user, ctx, self)?;
    Ok(loader_result_to_required_single!(loader_result, T))
  }
}

#[Object(guard = "UserConProfilePolicy::model_guard(UserConProfileAction::Read, self)")]
impl UserConProfileUsersFields {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "accepted_clickwrap_agreement")]
  async fn accepted_clickwrap_agreement(&self) -> bool {
    self.model.accepted_clickwrap_agreement
  }

  async fn address(&self) -> Option<&str> {
    self.model.address.as_deref()
  }

  async fn bio(&self) -> Option<&str> {
    self.model.bio.as_deref()
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

  #[graphql(name = "can_have_bio")]
  async fn can_have_bio(&self, ctx: &Context<'_>) -> Result<bool> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    Ok(
      authorization_info
        .is_user_con_profile_bio_eligible(self.model.id, self.model.convention_id)
        .await?,
    )
  }

  async fn city(&self) -> Option<&str> {
    self.model.city.as_deref()
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
    guard = "UserConProfilePolicy::model_guard(UserConProfileAction::ReadPersonalInfo, self)"
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

  #[graphql(name = "show_nickname_in_bio")]
  async fn show_nickname_in_bio(&self) -> bool {
    self.model.show_nickname_in_bio.unwrap_or(false)
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

  async fn state(&self) -> Option<&str> {
    self.model.state.as_deref()
  }

  #[graphql(name = "user_id")]
  async fn user_id(&self) -> ID {
    self.model.user_id.into()
  }

  async fn zipcode(&self) -> Option<&str> {
    self.model.zipcode.as_deref()
  }
}
