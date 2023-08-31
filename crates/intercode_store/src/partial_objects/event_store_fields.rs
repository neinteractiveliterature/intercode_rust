use std::sync::Arc;

use async_graphql::*;
use futures::StreamExt;
use intercode_entities::{events, maximum_event_provided_tickets_overrides};
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_graphql_loaders::LoaderManager;
use intercode_policies::{AuthorizationInfo, Policy, ReadManageAction};
use seawater::loaders::{ExpectModel, ExpectModels};

use crate::policies::MaximumEventProvidedTicketsOverridePolicy;

model_backed_type!(EventStoreFields, events::Model);

impl EventStoreFields {
  pub async fn maximum_event_provided_tickets_overrides(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<maximum_event_provided_tickets_overrides::Model>> {
    let loaders = ctx.data::<Arc<LoaderManager>>()?;
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let convention_result = loaders.event_convention().load_one(self.model.id).await?;
    let convention = convention_result.expect_one()?;
    let meptos_result = loaders
      .event_maximum_event_provided_tickets_overrides()
      .load_one(self.model.id)
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
      .map(|mepto| mepto.clone())
      .collect::<Vec<_>>()
      .await;
    Ok::<_, Error>(meptos)
  }
}
