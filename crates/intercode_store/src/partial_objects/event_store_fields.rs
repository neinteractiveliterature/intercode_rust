use std::sync::Arc;

use async_graphql::*;
use async_trait::async_trait;
use futures::StreamExt;
use intercode_entities::{events, maximum_event_provided_tickets_overrides, ticket_types};
use intercode_graphql_core::{load_one_by_model_id, loader_result_to_many, ModelBackedType};
use intercode_graphql_loaders::LoaderManager;
use intercode_policies::{AuthorizationInfo, Policy, ReadManageAction};
use seawater::loaders::{ExpectModel, ExpectModels};

use crate::policies::MaximumEventProvidedTicketsOverridePolicy;

#[async_trait]
pub trait EventStoreExtensions
where
  Self: ModelBackedType<Model = events::Model>,
{
  async fn maximum_event_provided_tickets_overrides<
    T: ModelBackedType<Model = maximum_event_provided_tickets_overrides::Model> + Send,
  >(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<T>> {
    let loaders = ctx.data::<Arc<LoaderManager>>()?;
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let convention_result = loaders
      .event_convention()
      .load_one(self.get_model().id)
      .await?;
    let convention = convention_result.expect_one()?;
    let meptos_result = loaders
      .event_maximum_event_provided_tickets_overrides()
      .load_one(self.get_model().id)
      .await?;
    let meptos = meptos_result.expect_models()?;

    let meptos_stream = futures::stream::iter(meptos);
    let readable_meptos = meptos_stream.filter(|mepto| {
      let mepto = (*mepto).clone();
      async {
        MaximumEventProvidedTicketsOverridePolicy::action_permitted(
          authorization_info,
          &ReadManageAction::Read,
          &(convention.clone(), self.get_model().clone(), mepto),
        )
        .await
        .unwrap_or(false)
      }
    });
    let meptos = readable_meptos
      .map(|mepto| T::new(mepto.clone()))
      .collect::<Vec<_>>()
      .await;
    Ok::<_, Error>(meptos)
  }

  async fn ticket_types<T: ModelBackedType<Model = ticket_types::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<T>> {
    let loader_result = load_one_by_model_id!(event_ticket_types, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, T))
  }
}
