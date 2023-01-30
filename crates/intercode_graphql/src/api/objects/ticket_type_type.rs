use async_graphql::*;
use intercode_entities::ticket_types;
use seawater::loaders::ExpectModels;

use crate::{model_backed_type, QueryData};

use super::{ModelBackedType, ProductType};
model_backed_type!(TicketTypeType, ticket_types::Model);

#[Object(name = "TicketType")]
impl TicketTypeType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn description(&self) -> Option<&str> {
    self.model.description.as_deref()
  }

  #[graphql(name = "maximum_event_provided_tickets")]
  async fn maximum_event_provided_tickets(&self) -> i32 {
    self.model.maximum_event_provided_tickets
  }

  async fn name(&self) -> &String {
    &self.model.name
  }

  #[graphql(name = "providing_products")]
  async fn providing_products(&self, ctx: &Context<'_>) -> Result<Vec<ProductType>, Error> {
    let query_data = ctx.data::<QueryData>()?;

    Ok(
      query_data
        .loaders()
        .ticket_type_providing_products()
        .load_one(self.model.id)
        .await?
        .expect_models()?
        .iter()
        .map(|product| ProductType::new(product.to_owned()))
        .collect(),
    )
  }
}
