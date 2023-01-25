use sea_orm::{
  sea_query::{Expr, Func},
  ColumnTrait, Condition, EntityTrait, QueryFilter, Select,
};

pub fn numbered_placeholders(start: usize, count: usize) -> String {
  (start..(start + count))
    .into_iter()
    .map(|index| format!("${}", index))
    .collect::<Vec<_>>()
    .join(", ")
}

pub fn string_search_condition(search_string: &str, column: impl ColumnTrait) -> Condition {
  let terms = search_string.split_whitespace().filter_map(|term| {
    if !term.is_empty() {
      Some(format!("%{}%", term.to_lowercase()))
    } else {
      None
    }
  });

  terms.fold(Condition::any(), |cond, term| {
    let lower_col = Expr::expr(Func::lower(Expr::col(column)));
    cond.add(lower_col.like(term))
  })
}

pub fn string_search<T: EntityTrait>(
  scope: Select<T>,
  search_string: &str,
  column: impl ColumnTrait,
) -> Select<T> {
  scope.filter(string_search_condition(search_string, column))
}
