use std::borrow::Cow;
use std::collections::HashSet;
use std::sync::Arc;

use async_graphql::parser::types::Field;
use async_graphql::registry::{MetaField, MetaType, MetaTypeId, Registry};
use async_graphql::{
  ContainerType, Context, ContextSelectionSet, Error, InterfaceType, OutputType, Positioned,
};
use async_graphql_value::indexmap::IndexMap;
use async_trait::async_trait;
use intercode_entities::model_ext::form_item_permissions::FormItemRole;
use intercode_entities::model_ext::FormResponse;
use intercode_entities::{form_items, forms};
use intercode_graphql_core::scalars::JsonScalar;
use intercode_graphql_core::ModelBackedType;
use intercode_graphql_loaders::LoaderManager;
use intercode_inflector::IntercodeInflector;
use seawater::loaders::ExpectModels;

use crate::api::objects::{EventProposalType, EventType, UserConProfileType};
use crate::presenters::form_response_presenter::{
  attached_images_by_filename, form_response_as_json, FormResponsePresentationFormat,
};
use crate::SchemaData;

async fn load_filtered_form_items(
  loaders: &LoaderManager,
  form_id: i64,
  item_identifiers: Option<Vec<String>>,
) -> Result<Vec<form_items::Model>, Error> {
  let form_items_result = loaders.form_form_items().load_one(form_id).await?;
  let form_items = form_items_result.expect_models()?;
  let form_items: Vec<form_items::Model> = match item_identifiers {
    Some(item_identifiers) => {
      let item_identifiers: HashSet<String> = HashSet::from_iter(item_identifiers.into_iter());
      form_items
        .iter()
        .filter(|item| {
          item
            .identifier
            .as_ref()
            .map(|identifier| item_identifiers.contains(identifier))
            .unwrap_or(false)
        })
        .cloned()
        .collect()
    }
    None => form_items.to_vec(),
  };

  Ok(form_items)
}

pub enum FormResponseInterface {
  Event(EventType),
  EventProposal(EventProposalType),
  UserConProfile(UserConProfileType),
}

// Hacks: Interface doesn't support MergedObject, so instead we're going to declare EventProposal as the canonical
// implementation of this interface, and assume that the others implement everything EventProposal does.
// This loses some type safety but it does let us maintain compatibility with the Ruby version of the schema.
impl OutputType for FormResponseInterface {
  #[doc = " Type the name."]
  fn type_name() -> Cow<'static, str> {
    Cow::Borrowed("FormResponse")
  }

  #[doc = " Create type information in the registry and return qualified typename."]
  fn create_type_info(registry: &mut Registry) -> String {
    registry.create_output_type::<Self, _>(MetaTypeId::Interface, |registry| {
      let mut fields: IndexMap<String, MetaField> = Default::default();
      let mut cache_control = ::std::default::Default::default();

      if let MetaType::Object {
        fields: obj_fields,
        cache_control: obj_cache_control,
        ..
      } = registry.create_fake_output_type::<EventProposalType>()
      {
        fields = obj_fields;
        cache_control = obj_cache_control;
      }

      MetaType::Object {
        name: Self::type_name().into_owned(),
        description: None,
        fields,
        cache_control,
        extends: false,
        shareable: true,
        inaccessible: false,
        tags: vec![],
        keys: None,
        visible: None,
        is_subscription: false,
        rust_typename: Some(std::any::type_name::<Self>()),
      }
    })
  }

  #[doc = " Resolve an output value to `async_graphql::Value`."]
  #[must_use]
  #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
  fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
    &'life0 self,
    ctx: &'life1 ContextSelectionSet<'life2>,
    field: &'life3 Positioned<Field>,
  ) -> std::pin::Pin<
    std::boxed::Box<
      (dyn futures::Future<
        Output = std::result::Result<async_graphql::Value, async_graphql::ServerError>,
      > + std::marker::Send
         + 'async_trait),
    >,
  >
  where
    'life0: 'async_trait,
    'life1: 'async_trait,
    'life2: 'async_trait,
    'life3: 'async_trait,
    Self: 'async_trait,
  {
    match self {
      FormResponseInterface::Event(event) => event.resolve(ctx, field),
      FormResponseInterface::EventProposal(event_proposal) => event_proposal.resolve(ctx, field),
      FormResponseInterface::UserConProfile(user_con_profile) => {
        user_con_profile.resolve(ctx, field)
      }
    }
  }
}

impl ContainerType for FormResponseInterface {
  #[doc = " Resolves a field value and outputs it as a json value"]
  #[doc = " `async_graphql::Value`."]
  #[doc = ""]
  #[doc = " If the field was not found returns None."]
  #[must_use]
  #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
  fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
    &'life0 self,
    ctx: &'life1 Context<'life2>,
  ) -> std::pin::Pin<
    std::boxed::Box<
      (dyn futures::Future<
        Output = std::result::Result<
          std::option::Option<async_graphql::Value>,
          async_graphql::ServerError,
        >,
      > + std::marker::Send
         + 'async_trait),
    >,
  >
  where
    'life0: 'async_trait,
    'life1: 'async_trait,
    'life2: 'async_trait,
    Self: 'async_trait,
  {
    match self {
      FormResponseInterface::Event(event) => event.resolve_field(ctx),
      FormResponseInterface::EventProposal(event_proposal) => event_proposal.resolve_field(ctx),
      FormResponseInterface::UserConProfile(user_con_profile) => {
        user_con_profile.resolve_field(ctx)
      }
    }
  }
}

impl InterfaceType for FormResponseInterface {}

#[async_trait]
pub trait FormResponseImplementation<M>
where
  Self: ModelBackedType<Model = M>,
  M: sea_orm::ModelTrait + FormResponse + Send + Sync,
{
  async fn get_form(&self, ctx: &Context<'_>) -> Result<forms::Model, Error>;
  async fn get_team_member_name(&self, ctx: &Context<'_>) -> Result<String, Error>;

  async fn current_user_form_item_viewer_role(
    &self,
    ctx: &Context<'_>,
  ) -> Result<FormItemRole, Error>;

  async fn current_user_form_item_writer_role(
    &self,
    ctx: &Context<'_>,
  ) -> Result<FormItemRole, Error>;

  async fn form_response_attrs_json(
    &self,
    ctx: &Context<'_>,
    item_identifiers: Option<Vec<String>>,
  ) -> Result<JsonScalar, Error> {
    let schema_data = ctx.data::<SchemaData>()?;
    let loaders = ctx.data::<Arc<LoaderManager>>()?;
    let form = self.get_form(ctx).await?;

    let model = self.get_model();
    let attached_images = attached_images_by_filename(model, loaders).await?;

    let viewer_role = self.current_user_form_item_viewer_role(ctx).await?;

    let form_items = load_filtered_form_items(loaders, form.id, item_identifiers).await?;

    Ok(JsonScalar(form_response_as_json(
      model,
      form_items.iter(),
      &attached_images,
      viewer_role,
      FormResponsePresentationFormat::Plain,
      &schema_data.language_loader,
      &IntercodeInflector::new().pluralize(&self.get_team_member_name(ctx).await?),
    )))
  }

  async fn form_response_attrs_json_with_rendered_markdown(
    &self,
    ctx: &Context<'_>,
    item_identifiers: Option<Vec<String>>,
  ) -> Result<JsonScalar, Error> {
    let schema_data = ctx.data::<SchemaData>()?;
    let loaders = ctx.data::<Arc<LoaderManager>>()?;
    let form = self.get_form(ctx).await?;

    let model = self.get_model();
    let attached_images = attached_images_by_filename(model, loaders).await?;

    let viewer_role = self.current_user_form_item_viewer_role(ctx).await?;

    let form_items = load_filtered_form_items(loaders, form.id, item_identifiers).await?;

    Ok(JsonScalar(form_response_as_json(
      model,
      form_items.iter(),
      &attached_images,
      viewer_role,
      FormResponsePresentationFormat::Html,
      &schema_data.language_loader,
      &IntercodeInflector::new().pluralize(&self.get_team_member_name(ctx).await?),
    )))
  }
}
