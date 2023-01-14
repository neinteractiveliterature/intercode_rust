use crate::{events, runs};
use chrono::NaiveDateTime;
use sea_orm::{sea_query::Expr, ColumnTrait, Condition, EntityTrait, QueryFilter, Select};

pub trait TimeBoundsSelectExt<E: EntityTrait> {
  fn between(self, start: Option<NaiveDateTime>, finish: Option<NaiveDateTime>) -> Select<E>;
}

impl TimeBoundsSelectExt<runs::Entity> for Select<runs::Entity> {
  fn between(
    self,
    start: Option<NaiveDateTime>,
    finish: Option<NaiveDateTime>,
  ) -> Select<runs::Entity> {
    self.filter(Expr::cust_with_values(
      "tsrange(?, ?, '[)') && timespan_tsrange",
      vec![start, finish],
    ))
  }
}

impl TimeBoundsSelectExt<events::Entity> for Select<events::Entity> {
  fn between(
    self,
    start: Option<NaiveDateTime>,
    finish: Option<NaiveDateTime>,
  ) -> Select<events::Entity> {
    self.filter(Condition::any().add(
      events::Column::Id.in_subquery(runs::Entity::find().between(start, finish).query().take()),
    ))
  }
}
