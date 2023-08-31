// This is a frankenstein monster combining the csrf and axum_csrf crates.  axum_csrf doesn't play nice with axum-sessions
// so I'm trying to create something that will let them work in tandem

use async_trait::async_trait;
use axum::{
  extract::FromRequestParts,
  http::{self, StatusCode},
  middleware::Next,
  response::{IntoResponse, IntoResponseParts, Response, ResponseParts},
};
use axum_extra::extract::{
  cookie::{Cookie, Expiration, SameSite},
  CookieJar,
};
use base64::Engine;
use csrf::{ChaCha20Poly1305CsrfProtection, CsrfCookie, CsrfProtection, CsrfToken};
use http::{request::Parts, Request};
use std::{
  borrow::Cow,
  fmt::{Debug, Display},
  sync::Arc,
  time::Duration,
};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct CsrfExtractionFailure(&'static str);

impl IntoResponseParts for CsrfExtractionFailure {
  type Error = (StatusCode, &'static str);

  fn into_response_parts(self, _res: ResponseParts) -> Result<ResponseParts, Self::Error> {
    Err((StatusCode::INTERNAL_SERVER_ERROR, self.0))
  }
}

impl IntoResponse for CsrfExtractionFailure {
  fn into_response(self) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, self.0).into_response()
  }
}

impl Display for CsrfExtractionFailure {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.0)
  }
}

impl std::error::Error for CsrfExtractionFailure {}

#[derive(Clone)]
pub struct CsrfConfig {
  /// CSRF Cookie lifespan
  pub(crate) lifespan: Duration,
  /// CSRF cookie name
  pub(crate) cookie_name: String,
  /// CSRF Token character length
  pub(crate) cookie_len: usize,
  /// Session cookie domain
  pub(crate) cookie_domain: Option<Cow<'static, str>>,
  /// Session cookie http only flag
  pub(crate) cookie_http_only: bool,
  /// Session cookie http only flag
  pub(crate) cookie_path: Cow<'static, str>,
  /// Resticts how Cookies are sent cross-site. Default is `SameSite::None`
  /// Only works if domain is also set.
  pub(crate) cookie_same_site: SameSite,
  /// Session cookie secure flag
  pub(crate) cookie_secure: bool,
  ///Encyption Key used to encypt cookies for confidentiality, integrity, and authenticity.
  pub(crate) protect: Arc<dyn CsrfProtection>,
}

impl Debug for CsrfConfig {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CsrfConfig")
      .field("lifespan", &self.lifespan)
      .field("cookie_name", &self.cookie_name)
      .field("cookie_len", &self.cookie_len)
      .field("cookie_domain", &self.cookie_domain)
      .field("cookie_http_only", &self.cookie_http_only)
      .field("cookie_path", &self.cookie_path)
      .field("cookie_same_site", &self.cookie_same_site)
      .field("cookie_secure", &self.cookie_secure)
      .finish_non_exhaustive()
  }
}

impl CsrfConfig {
  pub fn new(secret: &[u8; 64]) -> Self {
    let mut csrf_secret: [u8; 32] = Default::default();
    csrf_secret.clone_from_slice(&secret[0..32]);
    let protect = ChaCha20Poly1305CsrfProtection::from_key(csrf_secret);
    CsrfConfig {
      cookie_domain: None,
      cookie_http_only: true,
      cookie_len: 2048,
      cookie_name: "csrf-token".to_string(),
      cookie_path: Cow::from("/".to_string()),
      cookie_same_site: SameSite::Lax,
      cookie_secure: true,
      lifespan: Duration::from_secs(300),
      protect: Arc::new(protect),
    }
  }
}

/// This is the Token that is generated when a user is routed to a page.
/// If a Cookie exists then it will be used as the Token.
/// Otherwise a new one is made.
#[derive(Clone)]
pub struct CsrfData {
  token: CsrfToken,
  cookie: CsrfCookie,
  config: CsrfConfig,
  pub verified: bool,
}

/// this auto pulls a Cookies nd Generates the CsrfToken from the extensions
#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for CsrfData {
  type Rejection = CsrfExtractionFailure;

  async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
    let config = parts
      .extensions
      .get::<CsrfConfig>()
      .cloned()
      .ok_or(CsrfExtractionFailure("Can't extract CsrfConfig extension"))?;
    let engine = base64::engine::general_purpose::STANDARD;

    let jar = CookieJar::from_request_parts(parts, state).await.unwrap();
    let cookie: Option<Vec<u8>> = jar
      .get(&config.cookie_name)
      .map(|cookie| cookie.value())
      .and_then(|value| engine.decode(value).ok());
    let cookie = cookie.and_then(|value| config.protect.parse_cookie(&value).ok());

    let header: Option<Vec<u8>> = parts
      .headers
      .get("x-csrf-token")
      .and_then(|header_value| engine.decode(header_value.as_bytes()).ok());
    let token = header.and_then(|value| config.protect.parse_token(&value).ok());

    let verified = match (token.as_ref(), cookie.as_ref()) {
      (Some(token), Some(cookie)) => config.protect.verify_token_pair(token, cookie),
      _ => false,
    };

    let (token, cookie) = config
      .protect
      .generate_token_pair(
        cookie
          .and_then(|c| {
            let c = c.value();
            if c.len() < 64 {
              None
            } else {
              let mut buf = [0; 64];
              buf.copy_from_slice(c);
              Some(buf)
            }
          })
          .as_ref(),
        config.lifespan.as_secs().try_into().unwrap(),
      )
      .or(Err(CsrfExtractionFailure("Can't generate token pair")))?;

    Ok(CsrfData {
      token,
      cookie,
      config,
      verified,
    })
  }
}

impl CsrfData {
  ///Used to get the hashed Token to place within the form.
  pub fn authenticity_token(&self) -> String {
    self.token.b64_string()
  }

  pub fn build_cookie(&self) -> Cookie<'static> {
    let lifespan = OffsetDateTime::now_utc() + self.config.lifespan;

    let mut cookie_builder =
      Cookie::build(self.config.cookie_name.clone(), self.cookie.b64_string())
        .path(self.config.cookie_path.clone())
        .secure(self.config.cookie_secure)
        .http_only(self.config.cookie_http_only)
        .expires(Expiration::DateTime(lifespan));

    if let Some(domain) = &self.config.cookie_domain {
      cookie_builder = cookie_builder
        .domain(domain.clone())
        .same_site(self.config.cookie_same_site);
    }

    cookie_builder.finish()
  }
}

pub async fn csrf_middleware<B: Send>(
  token: CsrfData,
  jar: CookieJar,
  req: Request<B>,
  next: Next<B>,
) -> Result<Response, (StatusCode, String)> {
  let future = next.run(req);
  let response = future.await;
  let cookie = token.build_cookie();
  Ok((jar.add(cookie), response).into_response())
}

pub fn enforce_csrf(token: CsrfData) -> Result<(), StatusCode> {
  if token.verified {
    Ok(())
  } else {
    Err(StatusCode::FORBIDDEN)
  }
}
