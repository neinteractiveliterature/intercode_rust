use std::borrow::Cow;

use async_graphql::{
  indexmap::IndexMap,
  parser::types::Field,
  registry::{Deprecation, MetaField, MetaType, MetaTypeId, Registry},
  resolver_utils::resolve_container,
  CacheControl, ContainerType, Context, ContextSelectionSet, Error, ObjectType, OutputType,
  Positioned, ServerResult, Value,
};
use async_trait::async_trait;
use intercode_inflector::inflector::string::pluralize;
use sea_orm::{
  ConnectionTrait, EntityTrait, FromQueryResult, ModelTrait, Paginator, PaginatorTrait, Select,
  SelectModel, SelectorTrait,
};

use crate::{query_data::QueryData, ModelBackedType};

/// ModelPaginator provides a way to use offset-based pagination on a list of objects. This
/// is useful for UIs such as Intercode's table views, which provide a way to jump between numbered
/// pages.
///
/// Page numbers in ModelPaginator are 1-based (so, the first page is page 1, then page 2,
/// etc.) The number of items per page can be controlled via the per_page argument on paginated
/// fields. It defaults to 20, and can go up to 200.
///
/// Offset-based pagination is different from
/// [Relay's cursor-based pagination](https://relay.dev/graphql/connections.htm) that is more
/// commonly used in GraphQL APIs. We chose to go with an offset-based approach due to our UI
/// needs, but if a cursor-based approach is desirable in the future, we may also implement Relay
/// connections alongside our existing pagination fields.
pub struct ModelPaginator<Item: ModelBackedType> {
  scope: Select<<Item::Model as ModelTrait>::Entity>,
  page: u64,
  per_page: u64,
}

impl<Item: ModelBackedType> ModelPaginator<Item>
where
  Item::Model: FromQueryResult + Sync,
{
  pub fn paginator<'s, 'db, C: ConnectionTrait>(
    &'s self,
    db: &'db C,
  ) -> Paginator<'s, C, SelectModel<Item::Model>>
  where
    'db: 's,
  {
    self.scope.clone().into_model().paginate(db, self.per_page)
  }

  async fn entries(&self, ctx: &Context<'_>) -> Result<Vec<Item>, Error> {
    let db = ctx.data::<QueryData>()?.db();
    Ok(
      self
        .paginator(db)
        .fetch_page(self.page - 1) // sqlx uses 0-based pagination, intercode uses 1-based
        .await?
        .into_iter()
        .map(Item::new)
        .collect(),
    )
  }

  pub fn into_type<O: ModelBackedType<Model = Item::Model>>(self) -> ModelPaginator<O> {
    ModelPaginator {
      scope: self.scope,
      page: self.page,
      per_page: self.per_page,
    }
  }
}

#[async_trait]
impl<Item: ModelBackedType + OutputType> OutputType for ModelPaginator<Item>
where
  Item::Model: Send + Sync + FromQueryResult,
  <<Item::Model as ModelTrait>::Entity as EntityTrait>::Model: Sync,
{
  fn type_name() -> Cow<'static, str> {
    Cow::Owned(format!(
      "{}Pagination",
      pluralize::to_plural(&Item::type_name())
    ))
  }

  #[doc = " Create type information in the registry and return qualified typename."]
  fn create_type_info(registry: &mut Registry) -> String {
    let existing_type = registry.types.get(Item::type_name().as_ref());
    if existing_type.is_none() {
      Item::create_type_info(registry);
    }

    registry.create_output_type::<Self, _>(MetaTypeId::Object, |_registry| {
      let mut fields: IndexMap<String, MetaField> = Default::default();
      let cache_control = ::std::default::Default::default();

      fields.insert(
        "current_page".to_string(),
        MetaField {
          name: "current_page".to_string(),
          description: Some(
            "The number of the page currently being returned in this query".to_string(),
          ),
          args: IndexMap::default(),
          ty: "Int!".to_string(),
          deprecation: Deprecation::NoDeprecated,
          cache_control: CacheControl::default(),
          external: false,
          requires: None,
          provides: None,
          visible: None,
          shareable: true,
          inaccessible: false,
          tags: vec![],
          override_from: None,
          compute_complexity: None,
          directive_invocations: vec![],
        },
      );

      fields.insert(
        "entries".to_string(),
        MetaField {
          name: "entries".to_string(),
          description: None,
          args: IndexMap::default(),
          ty: format!("[{}!]!", Item::type_name()),
          deprecation: Deprecation::NoDeprecated,
          cache_control: CacheControl::default(),
          external: false,
          requires: None,
          provides: None,
          visible: None,
          shareable: true,
          inaccessible: false,
          tags: vec![],
          override_from: None,
          compute_complexity: None,
          directive_invocations: vec![],
        },
      );

      fields.insert(
        "per_page".to_string(),
        MetaField {
          name: "per_page".to_string(),
          description: Some(
            "The number of items per page currently being returned in this query".to_string(),
          ),
          args: IndexMap::default(),
          ty: "Int!".to_string(),
          deprecation: Deprecation::NoDeprecated,
          cache_control: CacheControl::default(),
          external: false,
          requires: None,
          provides: None,
          visible: None,
          shareable: true,
          inaccessible: false,
          tags: vec![],
          override_from: None,
          compute_complexity: None,
          directive_invocations: vec![],
        },
      );

      fields.insert(
        "total_entries".to_string(),
        MetaField {
          name: "total_entries".to_string(),
          description: Some(
            "The total number of items in the paginated list (across all pages)".to_string(),
          ),
          args: IndexMap::default(),
          ty: "Int!".to_string(),
          deprecation: Deprecation::NoDeprecated,
          cache_control: CacheControl::default(),
          external: false,
          requires: None,
          provides: None,
          visible: None,
          shareable: true,
          inaccessible: false,
          tags: vec![],
          override_from: None,
          compute_complexity: None,
          directive_invocations: vec![],
        },
      );

      fields.insert(
        "total_pages".to_string(),
        MetaField {
          name: "total_pages".to_string(),
          description: Some("The total number of pages in the paginated list".to_string()),
          args: IndexMap::default(),
          ty: "Int!".to_string(),
          deprecation: Deprecation::NoDeprecated,
          cache_control: CacheControl::default(),
          external: false,
          requires: None,
          provides: None,
          visible: None,
          shareable: true,
          inaccessible: false,
          tags: vec![],
          override_from: None,
          compute_complexity: None,
          directive_invocations: vec![],
        },
      );

      MetaType::Object {
        name: Self::type_name().into_owned(),
        description: None,
        fields,
        cache_control,
        extends: false,
        shareable: true,
        inaccessible: false,
        resolvable: true,
        tags: vec![],
        keys: None,
        visible: None,
        is_subscription: false,
        rust_typename: Some(std::any::type_name::<Self>()),
        directive_invocations: vec![],
        interface_object: false,
      }
    })
  }

  #[doc = " Resolve an output value to `async_graphql::Value`."]
  #[must_use]
  #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
  async fn resolve(
    &self,
    ctx: &ContextSelectionSet<'_>,
    _field: &Positioned<Field>,
  ) -> ServerResult<Value> {
    resolve_container(ctx, self).await
  }
}

#[async_trait]
impl<Item: ModelBackedType + OutputType> ContainerType for ModelPaginator<Item>
where
  Item::Model: Sync + FromQueryResult,
  <Item::Model as ModelTrait>::Entity: Sync,
  <<Item::Model as ModelTrait>::Entity as EntityTrait>::Model: Sync,
{
  #[doc = " Resolves a field value and outputs it as a json value"]
  #[doc = " `async_graphql::Value`."]
  #[doc = ""]
  #[doc = " If the field was not found returns None."]
  #[must_use]
  #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
  async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<Value>> {
    match ctx.item.node.name.node.as_str() {
      "current_page" => Ok(Some(self.page.into())),
      "entries" => {
        let values = self.entries(ctx).await;

        match values {
          Ok(values) => OutputType::resolve(
            &values,
            &ctx.with_selection_set(&ctx.item.node.selection_set),
            ctx.item,
          )
          .await
          .map(Some),
          Err(err) => Err(err.into_server_error(Default::default())),
        }
      }
      "per_page" => Ok(Some(self.per_page.into())),
      "total_entries" => self
        .total_entries(ctx)
        .await
        .map(|value| Some(value.into()))
        .map_err(|err| err.into_server_error(Default::default())),
      "total_pages" => self
        .total_pages(ctx)
        .await
        .map(|value| Some(value.into()))
        .map_err(|err| err.into_server_error(Default::default())),
      _ => Ok(None),
    }
  }
}

impl<Item: ModelBackedType + OutputType> ObjectType for ModelPaginator<Item>
where
  Item::Model: Sync + FromQueryResult,
  <Item::Model as ModelTrait>::Entity: Sync,
  <<Item::Model as ModelTrait>::Entity as EntityTrait>::Model: Sync,
{
}

impl<Item: ModelBackedType> PaginationImplementation<Item::Model> for ModelPaginator<Item>
where
  Item::Model: FromQueryResult + Sync,
{
  type Selector = SelectModel<Item::Model>;

  fn new(
    scope: Option<Select<<Item::Model as ModelTrait>::Entity>>,
    page: Option<u64>,
    per_page: Option<u64>,
  ) -> Self {
    Self {
      scope: scope.unwrap_or(<<Item::Model as ModelTrait>::Entity as EntityTrait>::find()),
      page: page.unwrap_or(1),
      per_page: per_page.unwrap_or(20),
    }
  }

  fn paginator_and_page_size<'s, C: ConnectionTrait>(
    &'s self,
    db: &'s C,
  ) -> (Paginator<'s, C, Self::Selector>, u64) {
    (self.paginator(db), self.per_page)
  }
}

#[async_trait]
pub trait PaginationImplementation<Model: ModelTrait + Send + Sync> {
  type Selector: SelectorTrait<Item = Model> + Send + Sync;

  fn new(scope: Option<Select<Model::Entity>>, page: Option<u64>, per_page: Option<u64>) -> Self;

  fn paginator_and_page_size<'s, C: ConnectionTrait>(
    &'s self,
    db: &'s C,
  ) -> (Paginator<'s, C, Self::Selector>, u64);

  async fn total_entries(&self, ctx: &Context) -> Result<u64, Error> {
    let db = ctx.data::<QueryData>()?.db();
    Ok(self.paginator_and_page_size(db).0.num_items().await?)
  }

  async fn total_pages(&self, ctx: &Context) -> Result<u64, Error> {
    let db = ctx.data::<QueryData>()?.db();
    Ok(self.paginator_and_page_size(db).0.num_pages().await?)
  }

  async fn current_page(&self, ctx: &Context) -> Result<u64, Error> {
    let db = ctx.data::<QueryData>()?.db();
    Ok(self.paginator_and_page_size(db).0.cur_page())
  }

  async fn per_page(&self, ctx: &Context) -> Result<u64, Error> {
    let db = ctx.data::<QueryData>()?.db();
    Ok(self.paginator_and_page_size(db).1)
  }
}
