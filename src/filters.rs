use intercode_entities::{cms_parent::CmsParent, conventions, root_sites};
use intercode_graphql::QueryData;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::{convert::Infallible, sync::Arc};
use tracing::log::warn;
use warp::{Filter, Rejection};

pub fn convention_by_request_host(
  db: Arc<DatabaseConnection>,
) -> impl Filter<Extract = (Option<conventions::Model>,), Error = Rejection> + Clone {
  warp::host::optional().and_then(move |authority: Option<warp::host::Authority>| {
    let db = db.clone();
    async move {
      let convention = match authority {
        Some(authority) => conventions::Entity::find()
          .filter(conventions::Column::Domain.eq(authority.host()))
          .one(db.as_ref())
          .await
          .unwrap_or_else(|error| {
            warn!("Error while querying for convention: {}", error);
            None
          }),
        None => None,
      };

      Ok::<_, Infallible>(convention)
    }
  })
}

pub fn cms_parent_from_convention(
  db: Arc<DatabaseConnection>,
) -> impl Filter<Extract = ((Option<conventions::Model>, CmsParent),), Error = Rejection> + Clone {
  convention_by_request_host(db.clone()).and_then(move |convention: Option<conventions::Model>| {
    let db = db.clone();
    async move {
      let cms_parent: CmsParent = if let Some(convention) = convention.clone() {
        convention.into()
      } else {
        root_sites::Entity::find()
          .one(db.as_ref())
          .await
          .map_err(|_| warp::reject())?
          .map(CmsParent::from)
          .ok_or_else(warp::reject)?
      };

      Ok::<_, Rejection>((convention, cms_parent))
    }
  })
}

pub fn query_data(
  db: Arc<DatabaseConnection>,
) -> impl Filter<Extract = (QueryData,), Error = Rejection> + Clone {
  cms_parent_from_convention(db)
    .and(warp::header::optional("X-Intercode-User-Timezone"))
    .map(
      |(convention, cms_parent): (Option<conventions::Model>, CmsParent),
       user_timezone: Option<String>| {
        let tz_name = if let Some(convention) = convention.as_ref() {
          if convention.timezone_mode == "convention_local" {
            convention.timezone_name.as_ref().or(user_timezone.as_ref())
          } else {
            user_timezone.as_ref()
          }
        } else {
          user_timezone.as_ref()
        };

        let timezone = tz_name
          .and_then(|tz_name| tz_name.parse::<chrono_tz::Tz>().ok())
          .unwrap_or(chrono_tz::Tz::UTC);

        (convention, cms_parent, timezone)
      },
    )
    .map(
      move |(convention, cms_parent, timezone): (
        Option<conventions::Model>,
        CmsParent,
        chrono_tz::Tz,
      )| {
        let cms_parent: Arc<CmsParent> = Arc::new(cms_parent);
        let convention = Arc::new(convention);
        QueryData::new(
          cms_parent,
          Arc::new(None),
          convention,
          timezone,
          Arc::new(None),
        )
      },
    )
}
