use crate::{active_storage_attachments, event_proposals, events, user_con_profiles};
use sea_orm::{ColumnTrait, EntityTrait, JsonValue, QueryFilter, Select};

pub trait FormResponse: Send + Sync {
  fn attached_images(&self) -> Select<active_storage_attachments::Entity>;
  fn get(&self, identifier: &str) -> Option<JsonValue>;
}

impl FormResponse for events::Model {
  fn attached_images(&self) -> Select<active_storage_attachments::Entity> {
    active_storage_attachments::Entity::find()
      .filter(active_storage_attachments::Column::RecordType.eq("Event"))
      .filter(active_storage_attachments::Column::RecordId.eq(self.id))
      .filter(active_storage_attachments::Column::Name.eq("image"))
  }

  fn get(&self, identifier: &str) -> Option<JsonValue> {
    match identifier {
      "title" => Some(JsonValue::String(self.title.clone())),
      "author" => self.author.to_owned().map(JsonValue::String),
      "email" => self.email.to_owned().map(JsonValue::String),
      // TODO event_email
      // "event_email" => self.event_,
      "team_mailing_list_name" => self
        .team_mailing_list_name
        .to_owned()
        .map(JsonValue::String),
      "organization" => self.organization.clone().map(JsonValue::String),
      "url" => self.url.clone().map(JsonValue::String),
      "length_seconds" => Some(JsonValue::Number(self.length_seconds.into())),
      "can_play_concurrently" => Some(JsonValue::Bool(self.can_play_concurrently)),
      "con_mail_destination" => self
        .con_mail_destination
        .to_owned()
        .map(JsonValue::String)
        ,
      "description" => self.description.clone().map(JsonValue::String),
      "short_blurb" => self.short_blurb.clone().map(JsonValue::String),
      "registration_policy" => self.registration_policy.to_owned(),
      "participant_communications" => self
        .participant_communications
        .to_owned()
        .map(JsonValue::String)
        ,
      // TODO age_restrictions
      // "age_restrictions" => self.,
      "age_restrictions_description" => self
        .age_restrictions_description
        .to_owned()
        .map(JsonValue::String)
        ,
      "minimum_age" => self
        .minimum_age
        .map(|minimum_age| JsonValue::Number(minimum_age.into())),
      "content_warnings" => self
        .content_warnings
        .to_owned()
        .map(JsonValue::String)
        ,
      _ => self
        .additional_info
        .as_ref()
        .and_then(|addl_info| addl_info.get(identifier).cloned()),
    }
  }
}

impl FormResponse for event_proposals::Model {
  fn attached_images(&self) -> Select<active_storage_attachments::Entity> {
    active_storage_attachments::Entity::find()
      .filter(active_storage_attachments::Column::RecordType.eq("EventProposal"))
      .filter(active_storage_attachments::Column::RecordId.eq(self.id))
      .filter(active_storage_attachments::Column::Name.eq("image"))
  }

  fn get(&self, identifier: &str) -> Option<JsonValue> {
    match identifier {
      // TODO event_proposal form response fields
      _ => self
        .additional_info
        .as_ref()
        .and_then(|addl_info| addl_info.get(identifier).cloned()),
    }
  }
}

impl FormResponse for user_con_profiles::Model {
  fn attached_images(&self) -> Select<active_storage_attachments::Entity> {
    active_storage_attachments::Entity::find()
      .filter(active_storage_attachments::Column::RecordType.eq("UserConProfile"))
      .filter(active_storage_attachments::Column::RecordId.eq(self.id))
      .filter(active_storage_attachments::Column::Name.eq("image"))
  }

  fn get(&self, identifier: &str) -> Option<JsonValue> {
    match identifier {
      // TODO user_con_profile form response fields
      _ => self
        .additional_info
        .as_ref()
        .and_then(|addl_info| addl_info.get(identifier).cloned()),
    }
  }
}
