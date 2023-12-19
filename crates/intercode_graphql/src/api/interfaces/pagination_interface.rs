use std::borrow::Cow;

use async_graphql::{
  parser::types::Field,
  registry::{MetaField, MetaType, MetaTypeId, Registry},
  ContainerType, Context, ContextSelectionSet, Description, InterfaceType, OutputType, Positioned,
};
use async_graphql_value::indexmap::IndexMap;
use indoc::indoc;
use intercode_email::objects::EmailRouteType;
use intercode_graphql_core::ModelPaginator;

use crate::api::merged_objects::{
  ConventionType, CouponType, EventProposalType, EventType, OrderType, SignupRequestType,
  SignupType, UserConProfileType, UserType,
};

#[allow(dead_code)]
pub enum PaginationInterface {
  ConventionsPagination(ModelPaginator<ConventionType>),
  CouponsPagination(ModelPaginator<CouponType>),
  EmailRoutesPagination(ModelPaginator<EmailRouteType>),
  EventProposalsPagination(ModelPaginator<EventProposalType>),
  EventsPagination(ModelPaginator<EventType>),
  OrdersPagination(ModelPaginator<OrderType>),
  SignupRequestsPagination(ModelPaginator<SignupRequestType>),
  SignupsPagination(ModelPaginator<SignupType>),
  UserConProfilesPagination(ModelPaginator<UserConProfileType>),
  UsersPagination(ModelPaginator<UserType>),
}

impl Description for PaginationInterface {
  fn description() -> &'static str {
    indoc! {"
      PaginationInterface provides a way to use offset-based pagination on a list of objects. This
      is useful for UIs such as Intercode's table views, which provide a way to jump between numbered
      pages.

      Page numbers in PaginationInterface are 1-based (so, the first page is page 1, then page 2,
      etc.) The number of items per page can be controlled via the per_page argument on paginated
      fields. It defaults to 20, and can go up to 200.

      Offset-based pagination is different from
      [Relay's cursor-based pagination](https://relay.dev/graphql/connections.htm) that is more
      commonly used in GraphQL APIs. We chose to go with an offset-based approach due to our UI
      needs, but if a cursor-based approach is desirable in the future, we may also implement Relay
      connections alongside our existing pagination fields.
    "}
  }
}

impl OutputType for PaginationInterface {
  fn type_name() -> Cow<'static, str> {
    Cow::Borrowed("PaginationInterface")
  }

  fn create_type_info(registry: &mut Registry) -> String {
    let possible_types: Vec<String> = vec![
      ModelPaginator::<ConventionType>::type_name().into_owned(),
      ModelPaginator::<CouponType>::type_name().into_owned(),
      ModelPaginator::<EmailRouteType>::type_name().into_owned(),
      ModelPaginator::<EventProposalType>::type_name().into_owned(),
      ModelPaginator::<EventType>::type_name().into_owned(),
      ModelPaginator::<OrderType>::type_name().into_owned(),
      ModelPaginator::<SignupRequestType>::type_name().into_owned(),
      ModelPaginator::<SignupType>::type_name().into_owned(),
      ModelPaginator::<UserConProfileType>::type_name().into_owned(),
      ModelPaginator::<UserType>::type_name().into_owned(),
    ];

    let output_type = registry.create_output_type::<Self, _>(MetaTypeId::Interface, |registry| {
      let mut fields: IndexMap<String, MetaField> = Default::default();

      if let MetaType::Object {
        fields: obj_fields, ..
      } = registry.create_fake_output_type::<ModelPaginator<CouponType>>()
      {
        fields = obj_fields;
        fields.remove("entries"); // the entries field differs between all paginated types
      }

      MetaType::Interface {
        name: Self::type_name().into_owned(),
        description: Some(Self::description().to_owned()),
        fields,
        possible_types: possible_types.iter().cloned().collect(),
        extends: false,
        inaccessible: false,
        tags: vec![],
        keys: None,
        visible: None,
        rust_typename: Some(std::any::type_name::<Self>()),
      }
    });

    for possible_type in possible_types {
      registry.add_implements(&possible_type, Self::type_name().as_ref());
    }

    output_type
  }

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
      PaginationInterface::ConventionsPagination(pagination) => pagination.resolve(ctx, field),
      PaginationInterface::CouponsPagination(pagination) => pagination.resolve(ctx, field),
      PaginationInterface::EmailRoutesPagination(pagination) => pagination.resolve(ctx, field),
      PaginationInterface::EventProposalsPagination(pagination) => pagination.resolve(ctx, field),
      PaginationInterface::EventsPagination(pagination) => pagination.resolve(ctx, field),
      PaginationInterface::OrdersPagination(pagination) => pagination.resolve(ctx, field),
      PaginationInterface::SignupRequestsPagination(pagination) => pagination.resolve(ctx, field),
      PaginationInterface::SignupsPagination(pagination) => pagination.resolve(ctx, field),
      PaginationInterface::UserConProfilesPagination(pagination) => pagination.resolve(ctx, field),
      PaginationInterface::UsersPagination(pagination) => pagination.resolve(ctx, field),
    }
  }
}

impl ContainerType for PaginationInterface {
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
      PaginationInterface::ConventionsPagination(pagination) => pagination.resolve_field(ctx),
      PaginationInterface::CouponsPagination(pagination) => pagination.resolve_field(ctx),
      PaginationInterface::EmailRoutesPagination(pagination) => pagination.resolve_field(ctx),
      PaginationInterface::EventProposalsPagination(pagination) => pagination.resolve_field(ctx),
      PaginationInterface::EventsPagination(pagination) => pagination.resolve_field(ctx),
      PaginationInterface::OrdersPagination(pagination) => pagination.resolve_field(ctx),
      PaginationInterface::SignupRequestsPagination(pagination) => pagination.resolve_field(ctx),
      PaginationInterface::SignupsPagination(pagination) => pagination.resolve_field(ctx),
      PaginationInterface::UserConProfilesPagination(pagination) => pagination.resolve_field(ctx),
      PaginationInterface::UsersPagination(pagination) => pagination.resolve_field(ctx),
    }
  }
}

impl InterfaceType for PaginationInterface {}
