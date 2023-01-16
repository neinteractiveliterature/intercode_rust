use crate::{active_storage_attachments, event_proposals, events, user_con_profiles};
use sea_orm::{ColumnTrait, EntityTrait, JsonValue, QueryFilter, Select};

pub trait FormResponse: Send + Sync {
  type Entity: EntityTrait;

  fn get_id_column() -> <Self::Entity as EntityTrait>::Column
  where
    Self: Sized;
  fn attached_images_scope() -> Select<active_storage_attachments::Entity>
  where
    Self: Sized;

  fn get_id(&self) -> i64;
  fn get(&self, identifier: &str) -> Option<JsonValue>;
}

impl FormResponse for events::Model {
  type Entity = events::Entity;

  fn get_id_column() -> <Self::Entity as EntityTrait>::Column {
    events::Column::Id
  }

  fn get_id(&self) -> i64 {
    self.id
  }

  fn attached_images_scope() -> Select<active_storage_attachments::Entity> {
    active_storage_attachments::Entity::find()
      .filter(active_storage_attachments::Column::RecordType.eq("Event"))
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
  type Entity = event_proposals::Entity;

  fn get_id_column() -> <Self::Entity as EntityTrait>::Column {
    event_proposals::Column::Id
  }

  fn get_id(&self) -> i64 {
    self.id
  }

  fn attached_images_scope() -> Select<active_storage_attachments::Entity> {
    active_storage_attachments::Entity::find()
      .filter(active_storage_attachments::Column::RecordType.eq("EventProposal"))
      .filter(active_storage_attachments::Column::Name.eq("image"))
  }

  fn get(&self, identifier: &str) -> Option<JsonValue> {
    match identifier {
      "title" => self.title.clone().map(JsonValue::String),
      "email" => self.email.clone().map(JsonValue::String),
      // TODO event_email
      // "event_email" => ,
      // TODO age_restrictions
      // "age_restrictions" => ,
      "team_mailing_list_name" => self.team_mailing_list_name.clone().map(JsonValue::String),
      "length_seconds" => self
        .length_seconds
        .map(|length_seconds| JsonValue::Number(length_seconds.into())),
      "description" => self.description.clone().map(JsonValue::String),
      "short_blurb" => self.short_blurb.clone().map(JsonValue::String),
      "registration_policy" => self.registration_policy.clone(),
      "can_play_concurrently" => self.can_play_concurrently.map(JsonValue::Bool),
      "timeblock_preferences" => self.timeblock_preferences.clone(),
      _ => self
        .additional_info
        .as_ref()
        .and_then(|addl_info| addl_info.get(identifier).cloned()),
    }
  }
}

impl FormResponse for user_con_profiles::Model {
  type Entity = user_con_profiles::Entity;

  fn get_id_column() -> <Self::Entity as EntityTrait>::Column {
    user_con_profiles::Column::Id
  }

  fn get_id(&self) -> i64 {
    self.id
  }

  fn attached_images_scope() -> Select<active_storage_attachments::Entity> {
    active_storage_attachments::Entity::find()
      .filter(active_storage_attachments::Column::RecordType.eq("UserConProfile"))
      .filter(active_storage_attachments::Column::Name.eq("image"))
  }

  fn get(&self, identifier: &str) -> Option<JsonValue> {
    match identifier {
      "first_name" => Some(JsonValue::String(self.first_name.clone())),
      "last_name" => Some(JsonValue::String(self.last_name.clone())),
      "nickname" => self.nickname.clone().map(JsonValue::String),
      "birth_date" => self
        .birth_date
        .map(|date| JsonValue::String(date.to_string())),
      "address" => self.address.clone().map(JsonValue::String),
      "city" => self.city.clone().map(JsonValue::String),
      "state" => self.state.clone().map(JsonValue::String),
      "zipcode" => self.zipcode.clone().map(JsonValue::String),
      "country" => self.country.clone().map(JsonValue::String),
      "mobile_phone" => self.mobile_phone.clone().map(JsonValue::String),
      "allow_sms" => Some(JsonValue::Bool(self.allow_sms)),
      "day_phone" => self.day_phone.clone().map(JsonValue::String),
      "evening_phone" => self.evening_phone.clone().map(JsonValue::String),
      "best_call_time" => self.best_call_time.clone().map(JsonValue::String),
      "preferred_contact" => self.preferred_contact.clone().map(JsonValue::String),
      "receive_whos_free_emails" => Some(JsonValue::Bool(self.receive_whos_free_emails)),
      _ => self
        .additional_info
        .as_ref()
        .and_then(|addl_info| addl_info.get(identifier).cloned()),
    }
  }
}
