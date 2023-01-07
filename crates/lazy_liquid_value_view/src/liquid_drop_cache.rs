use once_cell::race::OnceBox;

pub trait LiquidDropCache {
  fn get_once_cell<T>(&self, field_name: &str) -> Option<&OnceBox<T>>
  where
    Self: Sized;
}
