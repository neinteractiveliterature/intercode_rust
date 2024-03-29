use std::sync::Arc;

use async_graphql::*;
use chrono::Utc;
use intercode_entities::{conventions, departments, organizations, user_activity_alerts};
use intercode_graphql_core::{
  enums::{EmailMode, ShowSchedule, SiteMode, TicketMode, TimezoneMode},
  lax_id::LaxId,
  load_one_by_model_id, loader_result_to_many, loader_result_to_optional_single, model_backed_type,
  objects::ActiveStorageAttachmentType,
  query_data::QueryData,
  scalars::{DateScalar, JsonScalar},
  ModelBackedType,
};
use intercode_graphql_loaders::LoaderManager;
use sea_orm::{
  prelude::{async_trait::async_trait, DateTimeUtc},
  ColumnTrait, ModelTrait, QueryFilter,
};

model_backed_type!(ConventionConventionsFields, conventions::Model);

#[async_trait]
pub trait ConventionConventionsExtensions
where
  Self: ModelBackedType<Model = conventions::Model>,
{
  async fn departments<T: ModelBackedType<Model = departments::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<T>> {
    let loader_result = load_one_by_model_id!(convention_departments, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, T))
  }

  async fn organization<T: ModelBackedType<Model = organizations::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<T>> {
    let loader_result = load_one_by_model_id!(convention_organization, ctx, self)?;
    Ok(loader_result_to_optional_single!(loader_result, T))
  }

  async fn user_activity_alert<T: ModelBackedType<Model = user_activity_alerts::Model>>(
    &self,
    ctx: &Context<'_>,
    id: ID,
  ) -> Result<T> {
    let query_data = ctx.data::<QueryData>()?;
    Ok(T::new(
      self
        .get_model()
        .find_related(user_activity_alerts::Entity)
        .filter(user_activity_alerts::Column::Id.eq(LaxId::parse(id)?))
        .one(query_data.db())
        .await?
        .ok_or_else(|| Error::new("UserActivityAlert not found"))?,
    ))
  }

  async fn user_activity_alerts<T: ModelBackedType<Model = user_activity_alerts::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<T>> {
    let loader_result = load_one_by_model_id!(convention_user_activity_alerts, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, T))
  }
}

#[Object]
impl ConventionConventionsFields {
  async fn name(&self) -> &str {
    self.model.name.as_deref().unwrap_or_default()
  }

  async fn canceled(&self) -> bool {
    self.model.canceled
  }

  #[graphql(name = "clickwrap_agreement")]
  async fn clickwrap_agreement(&self) -> Option<&str> {
    self.model.clickwrap_agreement.as_deref()
  }

  #[graphql(name = "created_at")]
  async fn created_at(&self) -> Result<Option<DateScalar>> {
    self.model.created_at.map(DateScalar::try_from).transpose()
  }

  async fn domain(&self) -> &str {
    self.model.domain.as_str()
  }

  #[graphql(name = "email_from")]
  async fn email_from(&self) -> &str {
    self.model.email_from.as_str()
  }

  #[graphql(name = "email_mode")]
  async fn email_mode(&self) -> Result<EmailMode> {
    self
      .model
      .email_mode
      .as_str()
      .try_into()
      .map_err(Error::from)
  }

  #[graphql(name = "ends_at")]
  async fn ends_at(&self) -> Option<DateScalar> {
    self
      .model
      .ends_at
      .map(|t| DateScalar(DateTimeUtc::from_naive_utc_and_offset(t, Utc)))
  }

  #[graphql(name = "event_mailing_list_domain")]
  async fn event_mailing_list_domain(&self) -> Option<&str> {
    self.model.event_mailing_list_domain.as_deref()
  }

  async fn favicon(&self, ctx: &Context<'_>) -> Result<Option<ActiveStorageAttachmentType>> {
    Ok(
      ctx
        .data::<Arc<LoaderManager>>()?
        .convention_favicon
        .load_one(self.model.id)
        .await?
        .and_then(|models| models.get(0).cloned())
        .map(ActiveStorageAttachmentType::new),
    )
  }

  async fn hidden(&self) -> bool {
    self.model.hidden
  }

  async fn language(&self) -> &str {
    self.model.language.as_str()
  }

  async fn location(&self) -> Option<JsonScalar> {
    self.model.location.as_ref().cloned().map(JsonScalar)
  }

  #[graphql(name = "maximum_tickets")]
  async fn maximum_tickets(&self) -> Option<i32> {
    self.model.maximum_tickets
  }

  #[graphql(name = "open_graph_image")]
  async fn open_graph_image(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<ActiveStorageAttachmentType>> {
    Ok(
      ctx
        .data::<Arc<LoaderManager>>()?
        .convention_open_graph_image
        .load_one(self.model.id)
        .await?
        .and_then(|models| models.get(0).cloned())
        .map(ActiveStorageAttachmentType::new),
    )
  }

  #[graphql(name = "show_event_list")]
  async fn show_event_list(&self) -> Result<ShowSchedule> {
    self
      .model
      .show_event_list
      .as_str()
      .try_into()
      .map_err(Error::from)
  }

  #[graphql(name = "show_schedule")]
  async fn show_schedule(&self) -> Result<ShowSchedule> {
    self
      .model
      .show_schedule
      .as_str()
      .try_into()
      .map_err(Error::from)
  }

  #[graphql(name = "site_mode")]
  async fn site_mode(&self) -> Result<SiteMode, Error> {
    self
      .model
      .site_mode
      .as_str()
      .try_into()
      .map_err(Error::from)
  }

  #[graphql(name = "starts_at")]
  async fn starts_at(&self) -> Option<DateScalar> {
    self
      .model
      .starts_at
      .map(|t| DateScalar(DateTimeUtc::from_naive_utc_and_offset(t, Utc)))
  }

  #[graphql(name = "ticket_mode")]
  async fn ticket_mode(&self) -> Result<TicketMode, Error> {
    self
      .model
      .ticket_mode
      .as_str()
      .try_into()
      .map_err(Error::from)
  }

  #[graphql(name = "ticket_name")]
  async fn ticket_name(&self) -> &str {
    self.model.ticket_name.as_str()
  }

  async fn ticket_name_plural(&self) -> String {
    intercode_inflector::inflector::Inflector::to_plural(self.model.ticket_name.as_str())
  }

  #[graphql(name = "timezone_mode")]
  async fn timezone_mode(&self) -> Result<TimezoneMode, Error> {
    self
      .model
      .timezone_mode
      .as_str()
      .try_into()
      .map_err(Error::from)
  }

  #[graphql(name = "timezone_name")]
  async fn timezone_name(&self) -> Option<&str> {
    self.model.timezone_name.as_deref()
  }

  #[graphql(name = "updated_at")]
  async fn updated_at(&self) -> Result<Option<DateScalar>> {
    self.model.updated_at.map(DateScalar::try_from).transpose()
  }
}
