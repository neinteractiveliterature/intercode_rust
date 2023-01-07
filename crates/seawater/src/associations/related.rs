use std::marker::PhantomData;

use super::{
  value_adapters::{
    ModelBackedDropAssociationManyAdapter, ModelBackedDropAssociationOneOptionalAdapter,
    ModelBackedDropAssociationOneRequiredAdapter, ModelBackedDropAssociationValueAdapter,
  },
  AssociationMeta, TargetType,
};
use crate::{ContextContainer, ModelBackedDrop};
use liquid::ValueView;

pub struct RelatedModelBackedDropAssociation<
  FromDrop: ModelBackedDrop + ContextContainer,
  ToDrop: ModelBackedDrop,
  Value: ValueView,
> {
  meta: AssociationMeta<FromDrop, ToDrop>,
  value_adapter: Box<dyn ModelBackedDropAssociationValueAdapter<ToDrop = ToDrop, Value = Value>>,
}

impl<FromDrop: ModelBackedDrop + ContextContainer, ToDrop: ModelBackedDrop, Value: ValueView>
  RelatedModelBackedDropAssociation<FromDrop, ToDrop, Value>
{
  pub fn new(meta: AssociationMeta<FromDrop, ToDrop>) -> Self {
    let value_adapter: Box<
      dyn ModelBackedDropAssociationValueAdapter<ToDrop = ToDrop, Value = Value>,
    > = match meta.target_type {
      TargetType::OneOptional => Box::new(ModelBackedDropAssociationOneOptionalAdapter {
        _phantom: PhantomData::<ToDrop>,
      }),
      TargetType::OneRequired => Box::new(ModelBackedDropAssociationOneRequiredAdapter {
        _phantom: PhantomData::<ToDrop>,
      }),
      TargetType::Many => Box::new(ModelBackedDropAssociationManyAdapter {
        _phantom: PhantomData::<ToDrop>,
      }),
    };

    Self {
      meta,
      value_adapter,
    }
  }
}
