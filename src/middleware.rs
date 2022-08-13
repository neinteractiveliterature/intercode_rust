use axum::{
  async_trait,
  extract::{FromRequest, RequestParts},
};
use intercode_entities::{cms_parent::CmsParent, conventions, root_sites};
use intercode_graphql::{QueryData, SchemaData};
use regex::Regex;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::{convert::Infallible, sync::Arc};
use tracing::warn;

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
    let user_timezone = req
      .headers()
      .get("X-Intercode-User-Timezone")
      .and_then(|header| header.to_str().ok());

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
    let query_data = QueryData::new(
      cms_parent,
      Arc::new(None),
      convention,
      timezone,
      Arc::new(None),
    );

    Ok(Self(query_data))
  }
}
