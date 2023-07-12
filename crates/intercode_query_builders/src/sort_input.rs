use async_graphql::InputObject;
use sea_orm::Order;

#[derive(InputObject, Clone)]
/// A description of a field to sort a result set by. This is typically used in pagination
/// fields to specify how the results should be ordered.
pub struct SortInput {
  /// The name of the field to sort by.
  pub field: String,
  /// If true, the field will be sorted in descending order. If false, it will be sorted in
  /// ascending order.
  pub desc: bool,
}

impl SortInput {
  pub fn query_order(&self) -> Order {
    if self.desc {
      Order::Desc
    } else {
      Order::Asc
    }
  }
}
