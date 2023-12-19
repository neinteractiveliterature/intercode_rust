use async_graphql::*;
use intercode_entities::products;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_store::partial_objects::ProductStoreFields;

use crate::merged_model_backed_type;

use super::{product_variant_type::ProductVariantType, ticket_type_type::TicketTypeType};

model_backed_type!(ProductGlueFields, products::Model);

#[Object]
impl ProductGlueFields {
  #[graphql(name = "product_variants")]
  async fn product_variants(&self, ctx: &Context<'_>) -> Result<Vec<ProductVariantType>> {
    ProductStoreFields::from_type(self.clone())
      .product_variants(ctx)
      .await
      .map(|res| res.into_iter().map(ProductVariantType::from_type).collect())
  }

  #[graphql(name = "provides_ticket_type")]
  pub async fn provides_ticket_type(&self, ctx: &Context<'_>) -> Result<Option<TicketTypeType>> {
    ProductStoreFields::from_type(self.clone())
      .provides_ticket_type(ctx)
      .await
      .map(|opt| opt.map(TicketTypeType::from_type))
  }
}

merged_model_backed_type!(
  ProductType,
  products::Model,
  "Product",
  ProductGlueFields,
  ProductStoreFields
);
