use async_graphql::*;
use intercode_entities::{maximum_event_provided_tickets_overrides, ticket_types};
use intercode_graphql_core::{
  lax_id::LaxId, load_one_by_model_id, loader_result_to_many, model_backed_type,
  query_data::QueryData,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use super::ProductType;
model_backed_type!(TicketTypeType, ticket_types::Model);

#[Object(name = "TicketType")]
impl TicketTypeType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "allows_event_signups")]
  async fn allows_event_signups(&self) -> bool {
    self.model.allows_event_signups
  }

  #[graphql(name = "counts_towards_convention_maximum")]
  async fn counts_towards_convention_maximum(&self) -> bool {
    self.model.counts_towards_convention_maximum
  }

  async fn description(&self) -> Option<&str> {
    self.model.description.as_deref()
  }

  #[graphql(name = "maximum_event_provided_tickets")]
  async fn maximum_event_provided_tickets(
    &self,
    ctx: &Context<'_>,
    event_id: Option<ID>,
  ) -> Result<i32> {
    if let Some(event_id) = event_id {
      let db = ctx.data::<QueryData>()?.db();
      let mepto = maximum_event_provided_tickets_overrides::Entity::find()
        .filter(
          maximum_event_provided_tickets_overrides::Column::EventId.eq(LaxId::parse(event_id)?),
        )
        .one(db)
        .await?;

      if let Some(mepto) = mepto {
        return Ok(mepto.override_value);
      }
    }

    Ok(self.model.maximum_event_provided_tickets)
  }

  async fn name(&self) -> &String {
    &self.model.name
  }

  #[graphql(name = "providing_products")]
  async fn providing_products(&self, ctx: &Context<'_>) -> Result<Vec<ProductType>, Error> {
    let loader_result = load_one_by_model_id!(ticket_type_providing_products, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, ProductType))
  }
}
