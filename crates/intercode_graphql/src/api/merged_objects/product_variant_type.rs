use async_graphql::*;
use intercode_entities::product_variants;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_store::partial_objects::ProductVariantStoreFields;

use crate::merged_model_backed_type;

use super::product_type::ProductType;

model_backed_type!(ProductVariantGlueFields, product_variants::Model);

#[Object]
impl ProductVariantGlueFields {
  pub async fn product(&self, ctx: &Context<'_>) -> Result<ProductType> {
    ProductVariantStoreFields::from_type(self.clone())
      .product(ctx)
      .await
      .map(ProductType::from_type)
  }
}

merged_model_backed_type!(
  ProductVariantType,
  product_variants::Model,
  "ProductVariant",
  ProductVariantGlueFields,
  ProductVariantStoreFields
);
