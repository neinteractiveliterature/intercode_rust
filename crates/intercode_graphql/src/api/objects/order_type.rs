use async_graphql::*;
use intercode_entities::orders;

use crate::{loaders::expect::ExpectModels, model_backed_type, SchemaData};

use super::{ModelBackedType, OrderEntryType};
model_backed_type!(OrderType, orders::Model);

#[Object]
impl OrderType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "order_entries")]
  async fn order_entries(&self, ctx: &Context<'_>) -> Result<Vec<OrderEntryType>, Error> {
    let loader = &ctx.data::<SchemaData>()?.loaders.order_order_entries;

    Ok(
      loader
        .load_one(self.model.id)
        .await?
        .expect_models()?
        .iter()
        .map(|order_entry| OrderEntryType::new(order_entry.to_owned()))
        .collect(),
    )
  }
}
