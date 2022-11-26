use crate::csrf::{csrf_middleware, CsrfConfig, CsrfData};
use crate::db_sessions::DbSessionStore;
use crate::legacy_passwords::{verify_legacy_md5_password, verify_legacy_sha1_password};
use crate::liquid_renderer::IntercodeLiquidRenderer;
use crate::middleware::{AuthorizationInfoAndQueryDataFromRequest, QueryDataFromRequest};
use crate::Localizations;
use ::http::StatusCode;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::*;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::{Multipart, OriginalUri};
use axum::response::{self, IntoResponse};
use axum::routing::{get, post};
use axum::{Extension, Form, Router};
use axum_sessions::{SameSite, SessionHandle, SessionLayer};
use csrf::ChaCha20Poly1305CsrfProtection;
use futures_util::stream::StreamExt;
use i18n_embed::fluent::fluent_language_loader;
use i18n_embed::LanguageLoader;
use intercode_entities::cms_parent::CmsParentTrait;
use intercode_entities::{events, users};
use intercode_graphql::cms_rendering_context::CmsRenderingContext;
use intercode_graphql::{api, build_intercode_graphql_schema, LiquidRenderer, SchemaData};
use liquid::object;
use opentelemetry::global::shutdown_tracer_provider;
use regex::Regex;
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter};
use seawater::ConnectionWrapper;
use serde::Deserialize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::env;
use std::future::ready;
use std::io::BufReader;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tls_listener::TlsListener;
use tokio_rustls::TlsAcceptor;
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

async fn single_page_app_entry(
  OriginalUri(url): OriginalUri,
  schema_data: Extension<SchemaData>,
  AuthorizationInfoAndQueryDataFromRequest(authorization_info, query_data): AuthorizationInfoAndQueryDataFromRequest,
) -> Result<impl IntoResponse, ::http::StatusCode> {
  let event_path_regex: regex::Regex =
    Regex::new("^/events/(\\d+)").map_err(|_| ::http::StatusCode::INTERNAL_SERVER_ERROR)?;
  let db = &query_data.db;
  let path = url.path();
  let page_scope = query_data.cms_parent.cms_page_for_path(path);

  let page = if let Some(page_scope) = page_scope {
    page_scope
      .one(db.as_ref())
      .await
      .map_err(|_db_err| ::http::StatusCode::INTERNAL_SERVER_ERROR)?
  } else {
    None
  };

  let event = if let Some(convention) = query_data.convention.as_ref() {
    if convention.site_mode == "single_event" {
      convention
        .find_related(events::Entity)
        .one(db.as_ref())
        .await
        .map_err(|_db_err| ::http::StatusCode::INTERNAL_SERVER_ERROR)?
    } else if let Some(event_captures) = event_path_regex.captures(path) {
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

  let liquid_renderer =
    IntercodeLiquidRenderer::new(&query_data, &schema_data, Arc::new(authorization_info));

  let cms_rendering_context =
    CmsRenderingContext::new(object!({}), &query_data, Arc::new(liquid_renderer));
  let page_title = "TODO";

  Ok(response::Html(
    cms_rendering_context
      .render_app_root_content(&url, page_title, page.as_ref(), event.as_ref())
      .await,
  ))
}

async fn graphql_handler(
  schema: Extension<IntercodeSchema>,
  schema_data: Extension<SchemaData>,
  req: GraphQLRequest,
  AuthorizationInfoAndQueryDataFromRequest(authorization_info, query_data): AuthorizationInfoAndQueryDataFromRequest,
) -> GraphQLResponse {
  let authorization_info = Arc::new(authorization_info);
  let liquid_renderer =
    IntercodeLiquidRenderer::new(&query_data, &schema_data, authorization_info.clone());
  let req = req
    .into_inner()
    .data(query_data)
    .data::<Arc<dyn LiquidRenderer>>(Arc::new(liquid_renderer))
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

async fn sign_in(
  token: CsrfData,
  form: Option<Form<SignInParams>>,
  multipart: Option<Multipart>,
  QueryDataFromRequest(query_data): QueryDataFromRequest,
  session: Extension<SessionHandle>,
) -> Result<impl IntoResponse, StatusCode> {
  enforce_csrf(token)?;

  let params = if let Some(form) = form {
    form
  } else {
    let mut mp = multipart.unwrap();
    let mut mp_params: HashMap<String, String> = Default::default();

    while let Some(field) = mp.next_field().await.unwrap() {
      let name = field.name().unwrap().to_string();
      let value = field.text().await.unwrap();
      mp_params.insert(name, value);
    }

    let mp_value = serde_json::to_value(mp_params).unwrap();
    Form(serde_json::from_value::<SignInParams>(mp_value).unwrap())
  };

  let user = users::Entity::find()
    .filter(users::Column::Email.eq(params.email.as_str()))
    .one(query_data.db.as_ref())
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
      .exec(query_data.db.as_ref())
      .await
      .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
  }

  let mut write_guard = session.write().await;
  write_guard
    .insert("current_user_id", user.id)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  Ok(response::Json(value!({ "status": "success" })))
}

pub async fn serve(db: DatabaseConnection) -> Result<()> {
  let db_conn = Arc::new(db);

  let language_loader = fluent_language_loader!();
  language_loader.load_languages(&Localizations, &[language_loader.fallback_language()])?;
  let language_loader_arc = Arc::new(language_loader);

  let schema_data = SchemaData {
    db_conn: db_conn.clone(),
    language_loader: Arc::clone(&language_loader_arc),
  };

  let graphql_schema = build_intercode_graphql_schema(schema_data.clone());

  let app = Router::new()
    .route("/graphql", get(graphql_playground).post(graphql_handler))
    .route("/authenticity_tokens", get(authenticity_tokens))
    .route("/users/sign_in", post(sign_in))
    .fallback(get(single_page_app_entry))
    .layer(axum_sea_orm_tx::Layer::new(ConnectionWrapper::from(
      schema_data.db_conn.clone(),
    )))
    .layer(Extension(schema_data))
    .layer(Extension(graphql_schema));

  let store = DbSessionStore::new(db_conn.into());
  let secret_bytes = hex::decode(env::var("SECRET_KEY_BASE")?)?;
  let secret: [u8; 64] = secret_bytes[0..64].try_into().unwrap_or_else(|_| {
    panic!(
      "SECRET_KEY_BASE is {} chars long but must be at least 128",
      secret_bytes.len() * 2
    )
  });
  let mut csrf_secret: [u8; 32] = Default::default();
  csrf_secret.clone_from_slice(&secret[0..32]);
  let protect = ChaCha20Poly1305CsrfProtection::from_key(csrf_secret);
  let session_layer = SessionLayer::new(store, &secret);
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

  let service = tower::ServiceBuilder::new()
    .concurrency_limit(
      env::var("MAX_CONCURRENCY")
        .unwrap_or_else(|_| "25".to_string())
        .parse()
        .unwrap_or(25),
    )
    .layer(CompressionLayer::new())
    .layer(Extension(csrf_config))
    .layer(axum::middleware::from_fn(csrf_middleware))
    .layer(session_layer)
    .layer(
      TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(DefaultOnResponse::new().level(tracing::Level::INFO)),
    )
    .layer(CatchPanicLayer::new())
    .service(app);

  let addr = SocketAddr::new(
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
    env::var("PORT")
      .unwrap_or_else(|_| String::from("5901"))
      .parse()?,
  );
  let signal = async move {
    tokio::signal::ctrl_c()
      .await
      .expect("failed to listen to shutdown signal");
  };

  if let (Ok(cert_path), Ok(key_path)) = (env::var("TLS_CERT_PATH"), env::var("TLS_KEY_PATH")) {
    let cert_pem = std::fs::File::open(cert_path)?;
    let key_pem = std::fs::File::open(key_path)?;
    let mut cert_reader = BufReader::new(cert_pem);
    let mut key_reader = BufReader::new(key_pem);

    let key = tokio_rustls::rustls::PrivateKey(
      rustls_pemfile::rsa_private_keys(&mut key_reader)?[0].to_owned(),
    );
    let certs = rustls_pemfile::certs(&mut cert_reader)?
      .iter()
      .map(|der| tokio_rustls::rustls::Certificate(der.to_owned()))
      .collect();

    let listener = tokio::net::TcpListener::bind(addr).await?;
    let acceptor: TlsAcceptor = Arc::new(
      tokio_rustls::rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .unwrap(),
    )
    .into();

    let incoming = TlsListener::new(acceptor, listener).filter(|conn| {
      if let Err(err) = conn {
        error!("Error: {:?}", err);
        ready(false)
      } else {
        ready(true)
      }
    });
    let shared = tower::make::Shared::new(service);

    hyper::Server::builder(hyper::server::accept::from_stream(incoming))
      .serve(shared)
      .with_graceful_shutdown(signal)
      .await?;
  } else {
    warn!(
      "TLS_CERT_PATH and/or TLS_KEY_PATH not present in env.  Falling back to unencrypted HTTP."
    );
    let listener = std::net::TcpListener::bind(addr)?;
    hyper::Server::from_tcp(listener)?
      .serve(tower::make::Shared::new(service))
      .with_graceful_shutdown(signal)
      .await?;
  };

  shutdown_tracer_provider();

  Ok(())
}
