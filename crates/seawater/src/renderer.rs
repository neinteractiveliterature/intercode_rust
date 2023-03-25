use crate::{Context, DropResult, DropResultTrait, DropStore};
use liquid::Parser;
use std::{
  fmt::Debug,
  pin::Pin,
  sync::{Arc, Weak},
};

pub trait BuildContextFn<C: Context>
where
  Self: Fn(Weak<DropStore<C::StoreID>>) -> C + Send + Sync,
{
}

impl<C: Context, T> BuildContextFn<C> for T where
  T: Fn(Weak<DropStore<C::StoreID>>) -> C + Send + Sync
{
}

pub trait BuildGlobalsFn<C: Context, G: liquid::ObjectView + Clone + DropResultTrait<G> + 'static>
where
  Self: Fn(C) -> G + Send + Sync,
{
}

impl<C: Context, G: liquid::ObjectView + Clone + DropResultTrait<G> + 'static, T>
  BuildGlobalsFn<C, G> for T
where
  T: Fn(C) -> G + Send + Sync,
{
}

pub struct Renderer<C: Context, G: liquid::ObjectView + Clone + DropResultTrait<G> + 'static> {
  parser: Parser,
  build_context: Pin<Box<dyn BuildContextFn<C>>>,
  build_globals: Pin<Box<dyn BuildGlobalsFn<C, G>>>,
}

impl<C: Context, G: liquid::ObjectView + Clone + DropResultTrait<G> + 'static> Debug
  for Renderer<C, G>
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Renderer")
      .field("context_type", &std::any::type_name::<C>())
      .finish_non_exhaustive()
  }
}

impl<C: Context, G: liquid::ObjectView + Clone + DropResultTrait<G> + 'static> Renderer<C, G> {
  pub fn new<BCF: BuildContextFn<C> + 'static, BGF: BuildGlobalsFn<C, G> + 'static>(
    parser: Parser,
    build_context: BCF,
    build_globals: BGF,
  ) -> Self {
    Self {
      parser,
      build_context: Box::pin(build_context),
      build_globals: Box::pin(build_globals),
    }
  }

  pub async fn render_liquid(
    &self,
    content: &str,
    globals: liquid::Object,
  ) -> Result<String, async_graphql::Error> {
    let store = DropStore::<C::StoreID>::new();
    let context = (self.build_context)(Arc::downgrade(&store));
    let builtin_globals = DropResult::new((self.build_globals)(context));
    let globals_with_builtins = builtin_globals.extend(globals);

    let template = self.parser.parse(content)?;
    let store_clone = store.clone();

    let result = tokio::task::spawn_blocking(move || {
      let result = template.render(&globals_with_builtins);

      drop(store_clone);
      result
    })
    .await?;

    match result {
      Ok(content) => Ok(content),
      Err(error) => Err(async_graphql::Error::new(error.to_string())),
    }
  }
}
