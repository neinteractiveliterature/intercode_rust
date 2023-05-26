use chrono::Utc;
use sea_orm::{
  prelude::DateTimeUtc, sea_query::Cond, ColumnTrait, EntityTrait, QueryFilter, QuerySelect, Select,
};

use crate::{conventions, event_categories, forms};

impl conventions::Model {
  pub fn all_forms(&self) -> Select<forms::Entity> {
    forms::Entity::find().filter(
      Cond::any()
        .add(forms::Column::ConventionId.eq(self.id))
        .add(
          forms::Column::Id.in_subquery(
            QuerySelect::query(
              &mut event_categories::Entity::find()
                .filter(event_categories::Column::ConventionId.eq(self.id))
                .select_only()
                .column(event_categories::Column::EventFormId),
            )
            .take(),
          ),
        )
        .add(
          forms::Column::Id.in_subquery(
            QuerySelect::query(
              &mut event_categories::Entity::find()
                .filter(event_categories::Column::ConventionId.eq(self.id))
                .select_only()
                .column(event_categories::Column::EventProposalFormId),
            )
            .take(),
          ),
        ),
    )
  }

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
