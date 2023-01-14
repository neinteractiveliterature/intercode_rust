use async_graphql::{Context, Error, Interface};
use async_session::async_trait;
use intercode_entities::forms;
use intercode_entities::model_ext::form_item_permissions::FormItemRole;
use intercode_entities::model_ext::FormResponse;
use intercode_inflector::IntercodeInflector;
use seawater::loaders::ExpectModels;

use crate::api::objects::{EventType, ModelBackedType};
use crate::api::scalars::JsonScalar;
use crate::presenters::form_response_presenter::{
  attached_images_by_filename, form_response_as_json, FormResponsePresentationFormat,
};
use crate::{QueryData, SchemaData};

#[derive(Interface)]
#[graphql(field(
  name = "form_response_attrs_json_with_rendered_markdown",
  type = "JsonScalar",
  arg(name = "itemIdentifiers", type = "Option<Vec<String>>")
))]
pub enum FormResponseInterface {
  Event(EventType),
}

#[async_trait]
pub trait FormResponseImplementation<M>
where
  Self: ModelBackedType<Model = M>,
  M: sea_orm::ModelTrait + FormResponse + Send + Sync,
{
  async fn get_form(&self, ctx: &Context<'_>) -> Result<forms::Model, Error>;
  async fn get_team_member_name(&self, ctx: &Context<'_>) -> Result<String, Error>;

  async fn form_response_attrs_json_with_rendered_markdown(
    &self,
    ctx: &Context<'_>,
    _item_identifiers: Option<Vec<String>>,
  ) -> Result<JsonScalar, Error> {
    let schema_data = ctx.data::<SchemaData>()?;
    let query_data = ctx.data::<QueryData>()?;
    let form = self.get_form(ctx).await?;

    // TODO handle item_identifiers properly
    let form_items_result = query_data.loaders.form_form_items.load_one(form.id).await?;
    let form_items = form_items_result.expect_models()?;

    let model = self.get_model();
    let attached_images = attached_images_by_filename(model, &query_data.db).await?;

    Ok(JsonScalar(form_response_as_json(
      model,
      form_items.iter(),
      &attached_images,
      // TODO get viewer_role from policy object
      FormItemRole::Normal,
      FormResponsePresentationFormat::Markdown,
      &schema_data.language_loader,
      &IntercodeInflector::new().pluralize(&self.get_team_member_name(ctx).await?),
    )))
  }
}
