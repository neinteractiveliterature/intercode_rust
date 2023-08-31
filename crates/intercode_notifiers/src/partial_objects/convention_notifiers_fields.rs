use std::sync::Arc;

use async_graphql::*;
use intercode_entities::conventions;
use intercode_graphql_core::{
  liquid_renderer::LiquidRenderer, model_backed_type, query_data::QueryData,
  schema_data::SchemaData,
};
use intercode_liquid_drops::drops::DropContext;
use intercode_policies::{
  policies::{ConventionAction, ConventionPolicy},
  ModelBackedTypeGuardablePolicy,
};
use seawater::DropStore;

use crate::build_notifier_preview;

model_backed_type!(ConventionNotifiersFields, conventions::Model);

#[Object]
impl ConventionNotifiersFields {
  /// Given a Liquid text string and a notification event, renders the Liquid to HTML using the
  /// current domain's CMS context as if it were the content for that notification type.
  #[graphql(
    name = "preview_notifier_liquid",
    guard = "ConventionPolicy::model_guard(ConventionAction::ViewReports, self)"
  )]
  async fn preview_notifier_liquid(
    &self,
    ctx: &Context<'_>,
    #[graphql(desc = "The key of the notification event to use for generating the preview.")]
    event_key: String,
    content: String,
  ) -> Result<String, Error> {
    let schema_data = ctx.data::<SchemaData>()?;
    let query_data = ctx.data::<QueryData>()?;
    let liquid_renderer = ctx.data::<Arc<dyn LiquidRenderer>>()?;
    let Some(convention) = query_data.convention() else {
      return Ok("".to_string());
    };

    let store = DropStore::new();
    let notifier = build_notifier_preview(
      convention,
      &event_key,
      DropContext::new(
        schema_data.clone(),
        query_data.clone_ref(),
        Arc::downgrade(&store),
      ),
    )?;
    notifier
      .render_content(&content, liquid_renderer.as_ref())
      .await
  }
}
