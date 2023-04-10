use std::fmt::Display;

use crate::coupons;
use rusty_money::{iso::Currency, Money};
use sea_orm::prelude::Decimal;

use super::orders::money_from_cents_and_currency;

#[derive(Clone, Debug)]
pub enum Discount {
  Fixed(Money<'static, Currency>),
  Percentage(Decimal),
  ProvidesProduct(i64),
}

impl Discount {
  pub fn discount_amount(
    &self,
    total_amount: Money<'static, Currency>,
  ) -> Option<Money<'static, Currency>> {
    match self {
      Discount::Fixed(amount) => Some(amount.clone()),
      Discount::Percentage(percentage) => Some(total_amount * (percentage / Decimal::from(100))),
      Discount::ProvidesProduct(_) => None,
    }
  }
}

#[derive(Clone, Debug)]
pub enum InvalidDiscount {
  NoDiscountTypeSpecified,
  MultipleDiscountTypesSpecified,
}

impl Display for InvalidDiscount {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      InvalidDiscount::NoDiscountTypeSpecified => f.write_str("No discount type specified"),
      InvalidDiscount::MultipleDiscountTypesSpecified => {
        f.write_str("Multiple discount types specified")
      }
    }
  }
}

impl coupons::Model {
  pub fn discount(&self) -> Result<Discount, InvalidDiscount> {
    let discount_options = vec![
      self.fixed_amount().map(Discount::Fixed),
      self.percent_discount.map(Discount::Percentage),
      self.provides_product_id.map(Discount::ProvidesProduct),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    if discount_options.len() == 1 {
      Ok(discount_options.get(0).unwrap().clone())
    } else if discount_options.is_empty() {
      Err(InvalidDiscount::NoDiscountTypeSpecified)
    } else {
      Err(InvalidDiscount::MultipleDiscountTypesSpecified)
    }
  }

  pub fn fixed_amount(&self) -> Option<Money<'static, Currency>> {
    money_from_cents_and_currency(
      self.fixed_amount_cents,
      self.fixed_amount_currency.as_deref(),
    )
  }
}
