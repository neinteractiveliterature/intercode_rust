use sea_orm::{
  sea_query::{Expr, SimpleExpr},
  EntityTrait, Order, QueryOrder, Select,
};

use crate::{event_proposals, events};

pub trait NormalizedTitle
where
  Self: EntityTrait,
{
  fn normalized_title() -> SimpleExpr;
}

pub trait OrderByTitle {
  fn order_by_title(self, ord: Order) -> Self;
}

impl<E: NormalizedTitle> OrderByTitle for Select<E> {
  fn order_by_title(self, ord: Order) -> Self {
    self.order_by(E::normalized_title(), ord)
  }
}

impl NormalizedTitle for events::Entity {
  fn normalized_title() -> SimpleExpr {
    Expr::cust(
      "regexp_replace(
        regexp_replace(
          trim(regexp_replace(unaccent(events.title), '[^0-9a-z ]', '', 'gi')),
          '^(the|a|an) +',
          '',
          'i'
        ),
        ' ',
        '',
        'g'
      )",
    )
  }
}

impl NormalizedTitle for event_proposals::Entity {
  fn normalized_title() -> SimpleExpr {
    Expr::cust(
      "regexp_replace(
        regexp_replace(
          trim(regexp_replace(unaccent(event_proposals.title), '[^0-9a-z ]', '', 'gi')),
          '^(the|a|an) +',
          '',
          'i'
        ),
        ' ',
        '',
        'g'
      )",
    )
  }
}
