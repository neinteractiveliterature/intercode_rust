use std::borrow::Cow;

use async_graphql::parser::types::Field;
use async_graphql::registry::{MetaField, MetaType, MetaTypeId, Registry};
use async_graphql::{
  ContainerType, Context, ContextSelectionSet, InterfaceType, OutputType, Positioned,
};
use async_graphql_value::indexmap::IndexMap;

use crate::api::objects::{EventProposalType, EventType, UserConProfileType};

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
