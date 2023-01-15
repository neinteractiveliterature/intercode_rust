use crate::{events, runs};
use chrono::NaiveDateTime;
use sea_orm::{
  sea_query::Expr, ColumnTrait, Condition, EntityTrait, QueryFilter, QuerySelect, Select,
};

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
      "tsrange($1, $2, '[)') && timespan_tsrange",
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
    self.filter(
      Condition::any().add(
        events::Column::Id.in_subquery(
          sea_orm::QuerySelect::query(
            &mut runs::Entity::find()
              .between(start, finish)
              .select_only()
              .column(runs::Column::EventId),
          )
          .take(),
        ),
      ),
    )
  }
}
