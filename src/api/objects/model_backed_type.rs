use sea_orm::ModelTrait;

pub trait ModelBackedType<M: ModelTrait> {
  fn new(model: M) -> Self;
}
