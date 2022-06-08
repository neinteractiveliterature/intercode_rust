use sea_orm::ModelTrait;

pub trait ModelBackedType<M: ModelTrait> {
  fn new(model: M) -> Self;
}

#[macro_export]
macro_rules! model_backed_type {
  ($type_name: ident, $model_type: ty) => {
    pub struct $type_name {
      model: $model_type,
    }

    impl ModelBackedType<$model_type> for $type_name {
      fn new(model: $model_type) -> Self {
        $type_name { model }
      }
    }
  };
}
