use std::{env, sync::Arc};

use html_escape::encode_double_quoted_attribute;
use intercode_entities::{
  active_storage_attachments, active_storage_blobs, conventions, events, pages,
};
use intercode_graphql::{QueryData, SchemaData};
use intercode_liquid::cms_parent_partial_source::PreloadPartialsStrategy;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde_json::json;

const NOSCRIPT_WARNING: &str = r#"
NOSCRIPT_WARNING = <<~HTML.html_safe
<noscript id="no-javascript-warning">
  <div class="container">
    <div class="alert alert-danger">
      <h2 class="mb-4">JavaScript disabled</h2>

      <div class="d-flex align-items-center">
        <h1 class="m-0 me-4">
          <i class="bi-exclamation-triangle-fill"></i>
        </h1>
        <div class="flex-grow-1">
          <p>
            Your web browser has JavaScript disabled.  This site is written mostly in JavaScript,
            and will not work without it.  Please enable JavaScript in your browser's settings (or
            disable your JavaScript-blocking browser extension for this site).
          </p>
        </div>
      </div>

      <div class="text-end">
        <a class="btn btn-primary" href=".">Reload page</a>
      </div>
    </div>
  </div>
</noscript>
HTML
"#;

// https://stackoverflow.com/questions/38461429/how-can-i-truncate-a-string-to-have-at-most-n-characters
fn truncate(s: &str, max_chars: usize) -> &str {
  match s.char_indices().nth(max_chars) {
    None => s,
    Some((idx, _)) => &s[..idx],
  }
}

fn url_with_possible_host(request_url: &url::Url, path: &str, host: Option<String>) -> String {
  if let Some(host) = host {
    url::Url::parse(format!("{}://{}{}", request_url.scheme(), host, path).as_str())
      .map(|url| url.to_string())
      .unwrap_or_else(|_| path.to_string())
  } else {
    path.to_string()
  }
}

fn assets_host_script() -> String {
  if let Ok(assets_host) = env::var("ASSETS_HOST") {
    format!(
      r#"
<script type="application/javascript">
    window.intercodeAssetsHost = {};
  </script>
"#,
      json!(assets_host),
    )
  } else {
    "".to_string()
  }
}

fn active_storage_blob_url(blob: &active_storage_blobs::Model) -> &str {
  // TODO figure out how we're going to handle generating URLs for AS blobs
  blob.key.as_str()
}

async fn find_blob_by_attached_model(
  db: Arc<DatabaseConnection>,
  record_type: &str,
  record_id: i64,
) -> Result<Option<active_storage_blobs::Model>, sea_orm::DbErr> {
  let result = active_storage_attachments::Entity::find()
    .filter(
      active_storage_attachments::Column::RecordType
        .eq(record_type)
        .and(active_storage_attachments::Column::RecordId.eq(record_id)),
    )
    .find_also_related(active_storage_blobs::Entity)
    .one(db.as_ref())
    .await?;

  if let Some((_attachment, Some(blob))) = result {
    Ok(Some(blob))
  } else {
    Ok(None)
  }
}

async fn open_graph_image_tag(
  db: Arc<DatabaseConnection>,
  convention: Option<&conventions::Model>,
) -> String {
  if let Some(convention) = convention {
    let blob = find_blob_by_attached_model(db, "Convention", convention.id)
      .await
      .unwrap_or(None);

    if let Some(blob) = blob {
      format!(
        r#"<meta property="og:image" content="{}">"#,
        active_storage_blob_url(&blob)
      )
    } else {
      "".to_string()
    }
  } else {
    "".to_string()
  }
}

fn cms_page_title<'a>(
  convention: Option<&'a conventions::Model>,
  page: &'a pages::Model,
) -> &'a str {
  if let Some(convention) = convention {
    if let Some(root_page_id) = convention.root_page_id {
      if root_page_id == page.id {
        return convention.name.as_deref().unwrap_or_default();
      }
    }
  }

  page.name.as_deref().unwrap_or_default()
}

async fn open_graph_meta_tags(
  db: Arc<DatabaseConnection>,
  convention: Option<&conventions::Model>,
  page: Option<&pages::Model>,
  event: Option<&events::Model>,
  page_title: &str,
  rendering_context: &CmsRenderingContext,
) -> String {
  let title_and_desc = if let Some(event) = event {
    let short_blurb = event.short_blurb.to_owned().unwrap_or_default();
    format!(
      r#"
<meta property="og:title" content="{}">
<meta property="og:description" content="{}">
"#,
      encode_double_quoted_attribute(event.title.as_str()),
      encode_double_quoted_attribute(short_blurb.as_str())
    )
  } else if let Some(page) = page {
    format!(
      r#"
<meta property="og:title" content="{}">
<meta property="og:description" content="{}">
"#,
      encode_double_quoted_attribute(cms_page_title(convention, page)),
      encode_double_quoted_attribute(truncate(
        rendering_context
          .render_liquid(
            page.content.as_deref().unwrap_or(""),
            Some(PreloadPartialsStrategy::ByPage(page))
          )
          .await
          .unwrap_or_else(|_| "".to_string())
          .trim(),
        160
      ))
    )
  } else {
    format!(
      r#"
<meta property="og:title" content="{}">
<meta property="og:description" content="">"#,
      page_title,
    )
  };

  format!(
    r#"{}\n{}\n<meta property="og:type" content="website">"#,
    open_graph_image_tag(db, convention).await,
    title_and_desc
  )
}

async fn convention_favicon_tag(
  db: Arc<DatabaseConnection>,
  convention: Option<&conventions::Model>,
) -> String {
  if let Some(convention) = convention {
    let blob = find_blob_by_attached_model(db, "Convention", convention.id)
      .await
      .unwrap_or(None);

    if let Some(blob) = blob {
      format!(
        r#"<link rel="icon" type="{}" href="{}">"#,
        encode_double_quoted_attribute(blob.content_type.as_deref().unwrap_or("")),
        active_storage_blob_url(&blob)
      )
    } else {
      "".to_string()
    }
  } else {
    "".to_string()
  }
}

async fn content_for_head(
  request_url: &url::Url,
  db: Arc<DatabaseConnection>,
  convention: Option<&conventions::Model>,
  page: Option<&pages::Model>,
  event: Option<&events::Model>,
  page_title: &str,
  rendering_context: &CmsRenderingContext,
) -> String {
  format!(
    r#"
<meta content="text/html; charset=UTF-8" http-equiv="Content-Type"/>
<title>{}</title>
{}
<script type="application/javascript" src="{}" type="module" defer>
<meta content="width=device-width, initial-scale=1" name="viewport"/>
<meta property="og:url" content="{}">
{}
{}
"#,
    page_title,
    assets_host_script(),
    url_with_possible_host(
      request_url,
      "/packs/application.js",
      env::var("ASSETS_HOST").ok()
    ),
    request_url.as_str(),
    open_graph_meta_tags(
      db.clone(),
      convention,
      page,
      event,
      page_title,
      rendering_context
    )
    .await,
    convention_favicon_tag(db, convention).await
  )
}

pub struct CmsRenderingContext {
  globals: liquid::Object,
  query_data: QueryData,
  schema_data: SchemaData,
}

impl CmsRenderingContext {
  pub fn new(
    globals: liquid::Object,
    schema_data: SchemaData,
    query_data: QueryData,
  ) -> Self {
    CmsRenderingContext {
      globals,
      query_data,
      schema_data,
    }
  }

  pub async fn render_liquid(
    &self,
    content: &str,
    preload_partials_strategy: Option<PreloadPartialsStrategy<'_>>,
  ) -> Result<String, async_graphql::Error> {
    self
      .query_data
      .render_liquid(
        &self.schema_data,
        content,
        self.globals.clone(),
        preload_partials_strategy,
      )
      .await
  }
}
