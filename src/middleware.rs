use axum::{
  async_trait,
  extract::{FromRequestParts, Host},
};
use axum_sessions::SessionHandle;
use http::{request::Parts, StatusCode};
use intercode_entities::{
  cms_parent::CmsParent, conventions, root_sites, user_con_profiles, users,
};
use intercode_graphql::{loaders::LoaderManager, QueryData};
use intercode_policies::AuthorizationInfo;
use regex::Regex;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use seawater::ConnectionWrapper;
use std::sync::Arc;
use tracing::{error, warn};

async fn convention_from_request_parts(
  parts: &mut Parts,
  db: &ConnectionWrapper,
) -> Option<conventions::Model> {
  let port_regex = Regex::new(":\\d+$").unwrap();
  let host_if_present = Host::from_request_parts(parts, &())
    .await
    .map(|host| port_regex.replace(&host.0, "").to_string())
    .ok();

  match host_if_present {
    Some(host) => conventions::Entity::find()
      .filter(conventions::Column::Domain.eq(host.to_lowercase()))
      .one(db.as_ref())
      .await
      .unwrap_or_else(|error| {
        warn!("Error while querying for convention: {}", error);
        None
      }),
    None => None,
  }
}

async fn cms_parent_from_request_parts(
  parts: &mut Parts,
  db: &ConnectionWrapper,
) -> Result<(Option<CmsParent>, Option<conventions::Model>), DbErr> {
  let convention = convention_from_request_parts(parts, db).await;

  let cms_parent = if let Some(convention) = &convention {
    Some(convention.clone().into())
  } else {
    root_sites::Entity::find()
      .one(db.as_ref())
      .await?
      .map(CmsParent::from)
  };

  Ok((cms_parent, convention))
}

pub struct QueryDataFromRequest(pub QueryData);

#[async_trait]
impl<S: Sync> FromRequestParts<S> for QueryDataFromRequest {
  type Rejection = (StatusCode, String);

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    let db = parts
      .extensions
      .get::<ConnectionWrapper>()
      .ok_or_else(|| {
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          "Could not get connection wrapper from request extensions".to_string(),
        )
      })?
      .clone();

    // let db = parts
    //   .extensions
    //   .get::<ConnectionWrapper>()
    //   .cloned()
    //   .ok_or_else(|| {
    //     (
    //       StatusCode::INTERNAL_SERVER_ERROR,
    //       "Could not get connection wrapper from request extensions".to_string(),
    //     )
    //   })?;

    let (cms_parent, convention) = cms_parent_from_request_parts(parts, &db)
      .await
      .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let session_handle = parts.extensions.get::<SessionHandle>().unwrap();
    let session = session_handle.read().await;
    let loader_manager = LoaderManager::new(db.clone());

    let Some(cms_parent) = cms_parent else {
    return Err((StatusCode::INTERNAL_SERVER_ERROR, "No root_site present in database".to_string()));
  };

    let user_timezone = parts
      .headers
      .get("X-Intercode-User-Timezone")
      .and_then(|header| header.to_str().ok());

    let current_user_id: Option<i64> = session.get("current_user_id");
    let current_user = Arc::new(if let Some(current_user_id) = current_user_id {
      users::Entity::find_by_id(current_user_id)
        .one(db.as_ref())
        .await
        .map_err(|db_err| {
          error!("Error finding current user: {:?}", db_err);
          (http::StatusCode::INTERNAL_SERVER_ERROR, db_err.to_string())
        })?
    } else {
      None
    });

    let tz_name = if let Some(convention) = convention.as_ref() {
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

    let user_con_profile = Arc::new(
      if let (Some(current_user), Some(convention)) = (current_user.as_ref(), convention.as_ref()) {
        user_con_profiles::Entity::find()
          .filter(user_con_profiles::Column::UserId.eq(current_user.id))
          .filter(user_con_profiles::Column::ConventionId.eq(convention.id))
          .one(db.as_ref())
          .await
          .map_err(|db_err| {
            error!("Error finding user_con_profile: {:?}", db_err);
            (http::StatusCode::INTERNAL_SERVER_ERROR, db_err.to_string())
          })?
      } else {
        None
      },
    );

    let query_data = QueryData {
      cms_parent: Arc::new(cms_parent),
      current_user,
      convention: Arc::new(convention),
      db: db.clone(),
      loaders: loader_manager,
      timezone,
      user_con_profile,
    };

    Ok(QueryDataFromRequest(query_data))
  }
}

pub struct AuthorizationInfoAndQueryDataFromRequest(pub AuthorizationInfo, pub QueryData);

#[async_trait]
impl<S: Sync> FromRequestParts<S> for AuthorizationInfoAndQueryDataFromRequest {
  type Rejection = http::StatusCode;

  async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
    let QueryDataFromRequest(query_data) = QueryDataFromRequest::from_request_parts(parts, state)
      .await
      .map_err(|(code, err)| {
        error!("{}", err);
        code
      })?;

    Ok(AuthorizationInfoAndQueryDataFromRequest(
      AuthorizationInfo::new(
        query_data.db.clone(),
        query_data.current_user.clone(),
        // TODO figure out how to get oauth scopes
        Default::default(),
        // TODO figure out how to do assumed identity stuff
        Arc::new(None),
      ),
      query_data,
    ))
  }
}
