use crate::csrf::{csrf_middleware, CsrfConfig, CsrfData};
use crate::db_sessions::SessionWithDbStoreFromTxLayer;
use crate::form_or_multipart::FormOrMultipart;
use crate::legacy_passwords::{verify_legacy_md5_password, verify_legacy_sha1_password};
use crate::liquid_renderer::IntercodeLiquidRenderer;
use crate::middleware::{AuthorizationInfoAndQueryDataFromRequest, QueryDataFromRequest};
use crate::request_bound_transaction::request_bound_transaction;
use crate::Localizations;
use ::http::StatusCode;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::*;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::body::HttpBody;
use axum::extract::{FromRef, OriginalUri, State};
use axum::middleware::from_fn_with_state;
use axum::response::{self, IntoResponse};
use axum::routing::{get, post, IntoMakeService};
use axum::{debug_handler, Extension, Form, Router};
use axum_server::tls_rustls::{RustlsAcceptor, RustlsConfig};
use axum_server::{Handle, Server};
use axum_sessions::{SameSite, SessionHandle};
use csrf::ChaCha20Poly1305CsrfProtection;
use i18n_embed::fluent::{fluent_language_loader, FluentLanguageLoader};
use i18n_embed::{I18nEmbedError, LanguageLoader};
use intercode_entities::cms_parent::CmsParentTrait;
use intercode_entities::{events, users};
use intercode_graphql::cms_rendering_context::CmsRenderingContext;
use intercode_graphql::{api, build_intercode_graphql_schema, LiquidRenderer, SchemaData};
use liquid::object;
use once_cell::sync::Lazy;
use opentelemetry::global::shutdown_tracer_provider;
use regex::Regex;
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter};
use serde::Deserialize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tower::limit::ConcurrencyLimitLayer;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::compression::CompressionLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::log::*;

const BCRYPT_COST: u32 = 10;

#[derive(Debug)]
struct FatalDatabaseError {
  #[allow(dead_code)]
  db_err: sea_orm::DbErr,
}

type IntercodeSchema = Schema<api::QueryRoot, EmptyMutation, EmptySubscription>;

#[derive(Clone, FromRef)]
pub struct AppState {
  schema: IntercodeSchema,
  schema_data: SchemaData,
  db_conn: Arc<DatabaseConnection>,
}

static EVENT_PATH_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("^/events/(\\d+)").unwrap());

#[debug_handler]
async fn single_page_app_entry(
  OriginalUri(url): OriginalUri,
  State(schema_data): State<SchemaData>,
  AuthorizationInfoAndQueryDataFromRequest(authorization_info, query_data): AuthorizationInfoAndQueryDataFromRequest,
) -> Result<impl IntoResponse, ::http::StatusCode> {
  let db = query_data.db();
  let path = url.path();
  let page_scope = query_data.cms_parent().cms_page_for_path(path);

  let page = if let Some(page_scope) = page_scope {
    page_scope
      .one(db.as_ref())
      .await
      .map_err(|_db_err| ::http::StatusCode::INTERNAL_SERVER_ERROR)?
  } else {
    None
  };

  let event = if let Some(convention) = query_data.convention() {
    if convention.site_mode == "single_event" {
      convention
        .find_related(events::Entity)
        .one(db.as_ref())
        .await
        .map_err(|_db_err| ::http::StatusCode::INTERNAL_SERVER_ERROR)?
    } else if let Some(event_captures) = EVENT_PATH_REGEX.captures(path) {
      let event_id = event_captures.get(1).unwrap().as_str().parse::<i64>();
      if let Ok(event_id) = event_id {
        convention
          .find_related(events::Entity)
          .filter(events::Column::Id.eq(event_id))
          .one(db.as_ref())
          .await
          .map_err(|_db_err| ::http::StatusCode::INTERNAL_SERVER_ERROR)?
      } else {
        None
      }
    } else {
      None
    }
  } else {
    None
  };

  let liquid_renderer = IntercodeLiquidRenderer::new(&query_data, &schema_data, authorization_info);

  let cms_rendering_context = CmsRenderingContext::new(object!({}), &query_data, &liquid_renderer);
  let page_title = "TODO";

  Ok(response::Html(
    cms_rendering_context
      .render_app_root_content(&url, page_title, page.as_ref(), event.as_ref())
      .await,
  ))
}

#[debug_handler(state = AppState)]
async fn graphql_handler(
  State(schema): State<IntercodeSchema>,
  State(schema_data): State<SchemaData>,
  AuthorizationInfoAndQueryDataFromRequest(authorization_info, query_data): AuthorizationInfoAndQueryDataFromRequest,
  req: GraphQLRequest,
) -> GraphQLResponse {
  let liquid_renderer =
    IntercodeLiquidRenderer::new(&query_data, &schema_data, authorization_info.clone());
  let req = req
    .into_inner()
    .data(query_data)
    .data::<Box<dyn LiquidRenderer>>(Box::new(liquid_renderer))
    .data(authorization_info);

  schema.execute(req).await.into()
}

async fn graphql_playground() -> impl IntoResponse {
  response::Html(playground_source(
    GraphQLPlaygroundConfig::new("/graphql").with_setting("schema.polling.interval", 10000),
  ))
}

async fn authenticity_tokens(token: CsrfData) -> impl IntoResponse {
  let value = token.authenticity_token();
  let response = vec![
    "graphql",
    "changePassword",
    "denyAuthorization",
    "grantAuthorization",
    "railsDirectUploads",
    "resetPassword",
    "signIn",
    "signOut",
    "signUp",
    "updateUser",
  ]
  .into_iter()
  .map(|field| (field, value.clone()))
  .collect::<HashMap<_, _>>();
  response::Json(response)
}

fn enforce_csrf(token: CsrfData) -> Result<(), StatusCode> {
  if token.verified {
    Ok(())
  } else {
    Err(StatusCode::FORBIDDEN)
  }
}

#[derive(Deserialize, Debug)]
struct SignInParams {
  #[serde(rename(deserialize = "user[email]"))]
  email: String,
  #[serde(rename(deserialize = "user[password]"))]
  password: String,
}

#[debug_handler]
async fn sign_in(
  token: CsrfData,
  QueryDataFromRequest(query_data): QueryDataFromRequest,
  session: Extension<SessionHandle>,
  form_or_multipart: FormOrMultipart<SignInParams>,
) -> Result<impl IntoResponse, StatusCode> {
  enforce_csrf(token)?;

  let params = match form_or_multipart {
    FormOrMultipart::Form(form) => form,
    FormOrMultipart::Multipart(mut multipart) => {
      let mut mp_params: HashMap<String, String> = Default::default();

      while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let value = field.text().await.unwrap();
        mp_params.insert(name, value);
      }

      let mp_value = serde_json::to_value(mp_params).unwrap();
      Form(serde_json::from_value::<SignInParams>(mp_value).unwrap())
    }
  };

  let user = users::Entity::find()
    .filter(users::Column::Email.eq(params.email.as_str()))
    .one(query_data.db().as_ref())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

  let password_matches = if !user.encrypted_password.is_empty() {
    bcrypt::verify(&params.password, &user.encrypted_password)
      .map_err(|_| StatusCode::NOT_ACCEPTABLE)?
  } else if let (Some(legacy_password_sha1), Some(legacy_password_sha1_salt)) =
    (user.legacy_password_sha1, user.legacy_password_sha1_salt)
  {
    verify_legacy_sha1_password(
      &params.password,
      &legacy_password_sha1,
      &legacy_password_sha1_salt,
    )
  } else if let Some(legacy_password_md5) = user.legacy_password_md5 {
    verify_legacy_md5_password(&params.password, &legacy_password_md5)
  } else {
    false
  };

  if !password_matches {
    return Err(StatusCode::NOT_ACCEPTABLE);
  }

  if user.encrypted_password.is_empty() {
    // upgrade the password while we have it in RAM
    let upgrade = users::ActiveModel {
      encrypted_password: ActiveValue::Set(bcrypt::hash(&params.password, BCRYPT_COST).unwrap()),
      legacy_password_md5: ActiveValue::Set(None),
      legacy_password_sha1: ActiveValue::Set(None),
      legacy_password_sha1_salt: ActiveValue::Set(None),
      ..Default::default()
    };

    users::Entity::update(upgrade)
      .exec(query_data.db().as_ref())
      .await
      .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
  }

  let mut write_guard = session.write().await;
  write_guard
    .insert("current_user_id", user.id)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  Ok(response::Json(value!({ "status": "success" })))
}

pub fn build_language_loader() -> Result<FluentLanguageLoader, I18nEmbedError> {
  let language_loader = fluent_language_loader!();
  language_loader.load_languages(&Localizations, &[language_loader.fallback_language()])?;

  Ok(language_loader)
}

async fn build_axum_server(
  addr: SocketAddr,
  handle: Handle,
) -> Result<Server<RustlsAcceptor>, Server> {
  if let (Ok(cert_path), Ok(key_path)) = (env::var("TLS_CERT_PATH"), env::var("TLS_KEY_PATH")) {
    let config = RustlsConfig::from_pem_file(cert_path, key_path)
      .await
      .map_err(|err| {
        warn!(
          "Falling back to unencrypted HTTP because of error reading cert and/or key: {}",
          err
        );
        axum_server::bind(addr).handle(handle.clone())
      })?;

    Ok(axum_server::bind_rustls(addr, config).handle(handle))
  } else {
    warn!(
      "TLS_CERT_PATH and/or TLS_KEY_PATH not present in env.  Falling back to unencrypted HTTP."
    );
    Err(axum_server::bind(addr).handle(handle))
  }
}

fn build_app<B: HttpBody + Send + Sync + Unpin + 'static>(
  db_conn: Arc<DatabaseConnection>,
) -> Result<IntoMakeService<Router<(), B>>, Error>
where
  axum::body::Bytes: From<<B as HttpBody>::Data>,
  <B as HttpBody>::Data: Send + Sync,
  <B as HttpBody>::Error: std::error::Error + Send + Sync,
{
  let language_loader_arc = Arc::new(build_language_loader()?);
  let schema_data = SchemaData {
    stripe_client: stripe::Client::new(env::var("STRIPE_SECRET_KEY")?),
    language_loader: language_loader_arc,
  };
  let graphql_schema = build_intercode_graphql_schema(schema_data.clone());

  let secret_bytes = hex::decode(env::var("SECRET_KEY_BASE")?)?;
  let secret: [u8; 64] = secret_bytes[0..64].try_into().unwrap_or_else(|_| {
    panic!(
      "SECRET_KEY_BASE is {} chars long but must be at least 128",
      secret_bytes.len() * 2
    )
  });

  let app_state = AppState {
    schema: graphql_schema,
    schema_data,
    db_conn,
  };

  let mut csrf_secret: [u8; 32] = Default::default();
  csrf_secret.clone_from_slice(&secret[0..32]);
  let protect = ChaCha20Poly1305CsrfProtection::from_key(csrf_secret);
  let session_layer = SessionWithDbStoreFromTxLayer::new(secret);
  let csrf_config = CsrfConfig {
    cookie_domain: None,
    cookie_http_only: true,
    cookie_len: 2048,
    cookie_name: "csrf-token".to_string(),
    cookie_path: Cow::from("/".to_string()),
    cookie_same_site: SameSite::Lax,
    cookie_secure: true,
    lifespan: Duration::from_secs(300),
    protect: Arc::new(protect),
  };

  let app = Router::new()
    .route("/graphql", get(graphql_playground).post(graphql_handler))
    .route("/authenticity_tokens", get(authenticity_tokens))
    .route("/users/sign_in", post(sign_in))
    .fallback(single_page_app_entry)
    .layer(axum::middleware::from_fn(csrf_middleware))
    .layer(Extension(csrf_config))
    .layer(session_layer)
    .layer(from_fn_with_state(
      app_state.clone(),
      request_bound_transaction,
    ))
    .layer(ConcurrencyLimitLayer::new(
      env::var("MAX_CONCURRENCY")
        .unwrap_or_else(|_| "25".to_string())
        .parse()
        .unwrap_or(25),
    ))
    .layer(CompressionLayer::new())
    .layer(
      TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(DefaultOnResponse::new().level(tracing::Level::INFO)),
    )
    .layer(CatchPanicLayer::new())
    .with_state(app_state)
    .into_make_service();
  Ok(app)
}

async fn graceful_shutdown(handle: Handle) {
  let duration = Duration::from_secs(
    env::var("SHUTDOWN_GRACE_PERIOD")
      .unwrap_or_else(|_| "30".to_string())
      .parse()
      .unwrap_or(30),
  );

  let immediate_shutdown_handle = handle.clone();
  tokio::spawn(async move {
    tokio::signal::ctrl_c().await.unwrap();
    info!("Received Ctrl-C; shutting down immediately");
    immediate_shutdown_handle.shutdown();
  });

  info!(
    "Starting graceful shutdown with {}-second timeout; press Ctrl-C again to shut down immediately",
    duration.as_secs()
  );

  handle.graceful_shutdown(Some(duration));

  while handle.connection_count() > 0 {
    sleep(Duration::from_secs(1)).await;
    info!("alive connections: {}", handle.connection_count());
  }
}

pub async fn serve(db: DatabaseConnection) -> Result<()> {
  let db_conn = Arc::new(db);
  let app = build_app::<hyper::Body>(db_conn)?;

  let addr = SocketAddr::new(
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
    env::var("PORT")
      .unwrap_or_else(|_| String::from("5901"))
      .parse()?,
  );

  let handle = Handle::new();
  let shutdown_handle = handle.clone();
  tokio::spawn(async move {
    tokio::signal::ctrl_c().await.unwrap();
    graceful_shutdown(shutdown_handle).await;
    info!("Shutting down server");
  });

  let server = build_axum_server(addr, handle).await;

  match server {
    Ok(server) => server.serve(app).await,
    Err(server) => server.serve(app).await,
  }?;

  shutdown_tracer_provider();

  Ok(())
}
