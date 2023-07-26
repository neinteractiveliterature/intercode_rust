use std::borrow::Cow;

use async_graphql::{
  parser::types::Field,
  registry::{MetaField, MetaType, MetaTypeId, Registry},
  ContainerType, Context, ContextSelectionSet, InterfaceType, OutputType, Positioned,
};
use async_graphql_value::indexmap::IndexMap;
use intercode_cms::api::partial_objects::ConventionCmsFields;

use crate::api::{merged_objects::RootSiteType, objects::ConventionType};

/// A CMS parent is a web site managed by Intercode. It acts as a container for CMS content, such
/// as pages, partials, files, layouts, variables, content groups, and user-defined GraphQL queries.
///
/// Most CMS parents are conventions, so their content will be convention-specific and scoped to
/// that convention's domain name. The exception to this is the root site, which is what Intercode
/// renders when there is no convention associated with the current domain name. (See the RootSite
/// object for more details about this.)
pub enum CmsParentInterface {
  RootSite(RootSiteType),
  Convention(ConventionType),
}

// Hacks: Interface doesn't support MergedObject, so instead we're going to declare ConventionCmsFields as the canonical
// implementation of this interface, and assume that RootSiteType implements everything ConventionCmsFields does.
// This loses some type safety but it does let us maintain compatibility with the Ruby version of the schema.
impl OutputType for CmsParentInterface {
  #[doc = " Type the name."]
  fn type_name() -> Cow<'static, str> {
    Cow::Borrowed("CmsParent")
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
      } = registry.create_fake_output_type::<ConventionCmsFields>()
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
      CmsParentInterface::RootSite(root_site) => root_site.resolve(ctx, field),
      CmsParentInterface::Convention(convention) => convention.resolve(ctx, field),
    }
  }
}

impl ContainerType for CmsParentInterface {
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
      CmsParentInterface::RootSite(root_site) => root_site.resolve_field(ctx),
      CmsParentInterface::Convention(convention) => convention.resolve_field(ctx),
    }
  }
}

impl InterfaceType for CmsParentInterface {}
