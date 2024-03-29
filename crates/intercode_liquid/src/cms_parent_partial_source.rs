use intercode_entities::{
  cms_layouts, cms_layouts_partials,
  cms_parent::{CmsParent, CmsParentTrait},
  cms_partials, cms_partials_pages, pages,
};
use sea_orm::{
  ColumnTrait, ConnectionTrait, FromQueryResult, Linked, ModelTrait, QueryFilter, QuerySelect,
  RelationTrait,
};
use seawater::ConnectionWrapper;
use std::{
  collections::HashMap,
  fmt::Debug,
  sync::{Arc, Mutex},
};

use liquid::partials::PartialSource;
use tokio::runtime::Handle;
use tracing::log::warn;

#[derive(Debug)]
pub enum PreloadPartialsStrategy<'a> {
  ByLayout(&'a intercode_entities::cms_layouts::Model),
  ByName(Vec<&'a str>),
  ByPage(&'a intercode_entities::pages::Model),
}

#[derive(FromQueryResult, Debug)]
struct PartialNameAndContent {
  name: String,
  content: Option<String>,
}

struct PageToCmsPartials;

impl Linked for PageToCmsPartials {
  type FromEntity = pages::Entity;
  type ToEntity = cms_partials::Entity;

  fn link(&self) -> Vec<sea_orm::LinkDef> {
    vec![
      cms_partials_pages::Relation::Pages.def().rev(),
      cms_partials_pages::Relation::CmsPartials.def(),
    ]
  }
}

struct LayoutToCmsPartials;

impl Linked for LayoutToCmsPartials {
  type FromEntity = cms_layouts::Entity;
  type ToEntity = cms_partials::Entity;

  fn link(&self) -> Vec<sea_orm::LinkDef> {
    vec![
      cms_layouts_partials::Relation::CmsLayouts.def().rev(),
      cms_layouts_partials::Relation::CmsPartials.def(),
    ]
  }
}

#[derive(Debug)]
struct LazyCmsPartialCacheCell {
  name: String,
  partial: Option<PartialNameAndContent>,
}

impl LazyCmsPartialCacheCell {
  fn new(name: String) -> LazyCmsPartialCacheCell {
    LazyCmsPartialCacheCell {
      name,
      partial: None,
    }
  }

  fn try_get<C: ConnectionTrait + Send + 'static>(
    &mut self,
    db: Arc<C>,
    cms_parent: &CmsParent,
  ) -> Option<&PartialNameAndContent> {
    if self.partial.is_some() {
      self.partial.as_ref()
    } else {
      let name = self.name.clone();
      warn!("Uncached single partial read: {}", name);
      let partials = cms_parent.cms_partials();
      let join_handle = tokio::spawn(async move {
        partials
          .filter(cms_partials::Column::Name.eq(name.as_str()))
          .select_only()
          .column(cms_partials::Column::Name)
          .column(cms_partials::Column::Content)
          .into_model::<PartialNameAndContent>()
          .one(db.as_ref())
          .await
          .ok()
          .unwrap_or_default()
      });
      let partial = Handle::current().block_on(join_handle).unwrap();
      self.partial = partial;
      self.partial.as_ref()
    }
  }

  fn preload(&mut self, partial: PartialNameAndContent) {
    self.partial = Some(partial);
  }
}

#[derive(Debug)]
struct LazyCmsPartialCache {
  cached_partials: Arc<Mutex<HashMap<String, LazyCmsPartialCacheCell>>>,
  db: ConnectionWrapper,
  cms_parent: CmsParent,
}

impl LazyCmsPartialCache {
  pub fn new(cms_parent: CmsParent, db: ConnectionWrapper) -> LazyCmsPartialCache {
    let cached_partials = HashMap::<String, LazyCmsPartialCacheCell>::new();

    LazyCmsPartialCache {
      cms_parent,
      db,
      cached_partials: Arc::new(Mutex::new(cached_partials)),
    }
  }

  fn contains(&self, name: &str) -> bool {
    self
      .cached_partials
      .lock()
      .map(|cache| cache.contains_key(name))
      .unwrap_or(false)
  }

  fn try_get(&self, name: &str) -> Option<String> {
    let mut partials = self.cached_partials.lock().unwrap();
    let cache_cell = partials
      .entry(name.to_string())
      .or_insert_with(|| LazyCmsPartialCacheCell::new(name.to_string()));
    if let Some(partial) = cache_cell.try_get(self.db.clone().into(), &self.cms_parent) {
      partial.content.clone()
    } else {
      None
    }
  }

  async fn preload_by_name(
    &self,
    db: &ConnectionWrapper,
    names: Vec<&str>,
  ) -> Result<(), sea_orm::DbErr> {
    let loaded_partials = self
      .cms_parent
      .cms_partials()
      .filter(cms_partials::Column::Name.is_in(names))
      .select_only()
      .column(cms_partials::Column::Name)
      .column(cms_partials::Column::Content)
      .into_model::<PartialNameAndContent>()
      .all(db)
      .await?;

    let mut partials = self.cached_partials.lock().unwrap();
    for partial in loaded_partials {
      partials
        .entry(partial.name.clone())
        .or_insert_with(|| LazyCmsPartialCacheCell::new(partial.name.clone()))
        .preload(partial);
    }
    Ok(())
  }

  async fn preload_by_layout(
    &self,
    db: &ConnectionWrapper,
    layout: &cms_layouts::Model,
  ) -> Result<(), sea_orm::DbErr> {
    let loaded_partials = layout
      .find_linked(LayoutToCmsPartials)
      .select_only()
      .column(cms_partials::Column::Name)
      .column(cms_partials::Column::Content)
      .into_model::<PartialNameAndContent>()
      .all(db)
      .await?;

    let mut partials = self.cached_partials.lock().unwrap();
    for partial in loaded_partials {
      partials
        .entry(partial.name.clone())
        .or_insert_with(|| LazyCmsPartialCacheCell::new(partial.name.clone()))
        .preload(partial);
    }
    Ok(())
  }

  async fn preload_by_page(
    &self,
    db: &ConnectionWrapper,
    page: &pages::Model,
  ) -> Result<(), sea_orm::DbErr> {
    let loaded_partials = page
      .find_linked(PageToCmsPartials)
      .select_only()
      .column(cms_partials::Column::Name)
      .column(cms_partials::Column::Content)
      .into_model::<PartialNameAndContent>()
      .all(db)
      .await?;

    let mut partials = self.cached_partials.lock().unwrap();
    for partial in loaded_partials {
      partials
        .entry(partial.name.clone())
        .or_insert_with(|| LazyCmsPartialCacheCell::new(partial.name.clone()))
        .preload(partial);
    }
    Ok(())
  }
}

#[derive(Debug)]
pub struct LazyCmsPartialSource {
  cache: LazyCmsPartialCache,
}

impl LazyCmsPartialSource {
  pub fn new(cms_parent: CmsParent, db: ConnectionWrapper) -> LazyCmsPartialSource {
    let cache = LazyCmsPartialCache::new(cms_parent, db);

    LazyCmsPartialSource { cache }
  }

  pub async fn preload<'a>(
    &self,
    db: &ConnectionWrapper,
    strategy: PreloadPartialsStrategy<'a>,
  ) -> Result<(), sea_orm::DbErr> {
    match strategy {
      PreloadPartialsStrategy::ByName(names) => self.cache.preload_by_name(db, names).await,
      PreloadPartialsStrategy::ByLayout(layout) => self.cache.preload_by_layout(db, layout).await,
      PreloadPartialsStrategy::ByPage(page) => self.cache.preload_by_page(db, page).await,
    }
  }
}

impl PartialSource for LazyCmsPartialSource {
  fn contains(&self, name: &str) -> bool {
    self.cache.contains(name)
  }

  // TODO: figure out if anything actually needs this
  // It doesn't seem possible for us to implement this since we need to use temporary values to calculate it and
  // therefore can't return borrowed strs
  // Looks like we might not really need this method to have an implementation as long as we don't use EagerCompiler
  fn names(&self) -> Vec<&str> {
    vec![]
  }

  fn try_get<'a>(&'a self, name: &str) -> Option<std::borrow::Cow<'a, str>> {
    self
      .cache
      .try_get(name)
      .map(|content| content.into())
      // Don't crash the rendering if an unknown partial is referenced
      .or_else(|| Some(format!("Unknown partial: {}", name).into()))
  }
}
