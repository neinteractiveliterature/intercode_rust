use axum::{
  async_trait,
  body::HttpBody,
  extract::{FromRequest, RequestParts},
};
use axum_sessions::SessionHandle;
use intercode_entities::{
  cms_parent::CmsParent, conventions, root_sites, user_con_profiles, users,
};
use intercode_graphql::{QueryData, SchemaData};
use intercode_policies::AuthorizationInfo;
use regex::Regex;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::{convert::Infallible, sync::Arc};
use tracing::{log::error, warn};

pub struct ConventionByRequestHost(pub Option<conventions::Model>);

#[async_trait]
impl<B> FromRequest<B> for ConventionByRequestHost
where
  B: Send,
{
  type Rejection = Infallible;

  async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
    let port_regex = Regex::new(":\\d+$").unwrap();
    let db = &req.extensions().get::<SchemaData>().unwrap().db;
    let host_if_present = req
      .headers()
      .get("host")
      .and_then(|host| host.to_str().ok())
      .map(|host| port_regex.replace(host, ""));

    let convention = match host_if_present {
      Some(host) => conventions::Entity::find()
        .filter(conventions::Column::Domain.eq(host.to_lowercase()))
        .one(db.as_ref())
        .await
        .unwrap_or_else(|error| {
          warn!("Error while querying for convention: {}", error);
          None
        }),
      None => None,
    };

    Ok(ConventionByRequestHost(convention))
  }
}

pub struct CmsParentFromRequest(pub CmsParent, pub Option<conventions::Model>);

#[async_trait]
impl<B> FromRequest<B> for CmsParentFromRequest
where
  B: Send,
{
  type Rejection = http::StatusCode;

  async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
    let convention_extractor = ConventionByRequestHost::from_request(req).await.unwrap();
    let db = &req.extensions().get::<SchemaData>().unwrap().db;

    let cms_parent: CmsParent = if let Some(convention) = &convention_extractor.0 {
      convention.clone().into()
    } else {
      root_sites::Entity::find()
        .one(db.as_ref())
        .await
        .map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR)?
        .map(CmsParent::from)
        .ok_or(http::StatusCode::INTERNAL_SERVER_ERROR)?
    };

    Ok(CmsParentFromRequest(cms_parent, convention_extractor.0))
  }
}

pub struct QueryDataFromRequest(pub QueryData);

#[async_trait]
impl<B> FromRequest<B> for QueryDataFromRequest
where
  B: Send,
{
  type Rejection = http::StatusCode;

  async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
    let cms_parent_extractor = CmsParentFromRequest::from_request(req).await?;
    let session_handle = req.extensions().get::<SessionHandle>().unwrap();
    let session = session_handle.read().await;
    let db = &req.extensions().get::<SchemaData>().unwrap().db;

    let user_timezone = req
      .headers()
      .get("X-Intercode-User-Timezone")
      .and_then(|header| header.to_str().ok());

    let current_user_id: Option<i64> = session.get("current_user_id");
    let current_user = Arc::new(if let Some(current_user_id) = current_user_id {
      users::Entity::find_by_id(current_user_id)
        .one(db.as_ref())
        .await
        .map_err(|db_err| {
          error!("Error finding current user: {:?}", db_err);
          http::StatusCode::INTERNAL_SERVER_ERROR
        })?
    } else {
      None
    });

    let tz_name = if let Some(convention) = cms_parent_extractor.1.as_ref() {
      if convention.timezone_mode == "convention_local" {
        convention.timezone_name.as_deref().or(user_timezone)
      } else {
        user_timezone
      }
    } else {
      user_timezone
    };

    let timezone = tz_name
      .and_then(|tz_name| tz_name.parse::<chrono_tz::Tz>().ok())
      .unwrap_or(chrono_tz::Tz::UTC);

    let cms_parent: Arc<CmsParent> = Arc::new(cms_parent_extractor.0);
    let convention = Arc::new(cms_parent_extractor.1);

    let user_con_profile = Arc::new(
      if let (Some(current_user), Some(convention)) = (current_user.as_ref(), convention.as_ref()) {
        user_con_profiles::Entity::find()
          .filter(user_con_profiles::Column::UserId.eq(current_user.id))
          .filter(user_con_profiles::Column::ConventionId.eq(convention.id))
          .one(db.as_ref())
          .await
          .map_err(|db_err| {
            error!("Error finding user_con_profile: {:?}", db_err);
            http::StatusCode::INTERNAL_SERVER_ERROR
          })?
      } else {
        None
      },
    );

    let query_data = QueryData::new(
      cms_parent,
      current_user,
      convention,
      session_handle.clone(),
      timezone,
      user_con_profile,
    );

    Ok(Self(query_data))
  }
}

pub struct AuthorizationInfoFromRequest(pub AuthorizationInfo);

#[async_trait]
impl<B: HttpBody + Send> FromRequest<B> for AuthorizationInfoFromRequest {
  type Rejection = http::StatusCode;

  async fn from_request(req: &mut axum::extract::RequestParts<B>) -> Result<Self, Self::Rejection> {
    let query_data = QueryDataFromRequest::from_request(req).await?.0;
    let schema_data = req
      .extensions()
      .get::<SchemaData>()
      .expect("SchemaData not found in request extensions");

    Ok(AuthorizationInfoFromRequest(AuthorizationInfo::new(
      schema_data.db.clone(),
      query_data.current_user,
      // TODO figure out how to get oauth scopes
      Default::default(),
      // TODO figure out how to do assumed identity stuff
      Arc::new(None),
    )))
  }
}
