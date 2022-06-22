use chrono::Utc;
use sea_orm::prelude::DateTimeUtc;

use crate::conventions;

impl conventions::Model {
  pub fn ended(&self) -> bool {
    if let Some(ends_at) = self.ends_at {
      DateTimeUtc::from_utc(ends_at, Utc) <= Utc::now()
    } else {
      false
    }
  }

  pub fn tickets_available_for_purchase(&self) -> bool {
    if self.ended() {
      return false;
    };

    if self.ticket_mode == "disabled" {
      return false;
    }

    // TODO products.ticket_providing.available.any? { |product| product.pricing_structure.price(time: Time.zone.now) }
    true
  }
}
