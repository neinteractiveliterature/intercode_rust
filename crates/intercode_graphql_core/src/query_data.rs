use std::{fmt::Debug, sync::Arc};

use intercode_entities::{cms_parent::CmsParent, conventions, user_con_profiles, users};
use seawater::ConnectionWrapper;

pub trait QueryDataContainer: Sync + Send + Debug {
  fn clone_ref(&self) -> Box<dyn QueryDataContainer>;
  fn cms_parent(&self) -> &CmsParent;
  fn current_user(&self) -> Option<&users::Model>;
  fn convention(&self) -> Option<&conventions::Model>;
  fn db(&self) -> &ConnectionWrapper;
  fn timezone(&self) -> &chrono_tz::Tz;
  fn user_con_profile(&self) -> Option<&user_con_profiles::Model>;
}

pub type QueryData = Box<dyn QueryDataContainer>;

impl Clone for QueryData {
  fn clone(&self) -> Self {
    self.clone_ref()
  }
}

#[derive(Debug)]
pub struct OwnedQueryData {
  pub cms_parent: CmsParent,
  pub current_user: Option<users::Model>,
  pub convention: Option<conventions::Model>,
  pub db: ConnectionWrapper,
  pub timezone: chrono_tz::Tz,
  pub user_con_profile: Option<user_con_profiles::Model>,
}

impl OwnedQueryData {
  pub fn new(
    cms_parent: CmsParent,
    current_user: Option<users::Model>,
    convention: Option<conventions::Model>,
    db: ConnectionWrapper,
    timezone: chrono_tz::Tz,
    user_con_profile: Option<user_con_profiles::Model>,
  ) -> Self {
    OwnedQueryData {
      cms_parent,
      current_user,
      convention,
      db,
      timezone,
      user_con_profile,
    }
  }
}

#[derive(Debug)]
pub struct ArcQueryData {
  owned_query_data: Arc<OwnedQueryData>,
}

impl ArcQueryData {
  pub fn new(owned_query_data: OwnedQueryData) -> Self {
    ArcQueryData {
      owned_query_data: Arc::new(owned_query_data),
    }
  }
}

impl QueryDataContainer for ArcQueryData {
  fn clone_ref(&self) -> Box<dyn QueryDataContainer> {
    Box::new(ArcQueryData {
      owned_query_data: self.owned_query_data.clone(),
    })
  }

  fn cms_parent(&self) -> &CmsParent {
    &self.owned_query_data.cms_parent
  }

  fn current_user(&self) -> Option<&users::Model> {
    self.owned_query_data.current_user.as_ref()
  }

  fn convention(&self) -> Option<&conventions::Model> {
    self.owned_query_data.convention.as_ref()
  }

  fn db(&self) -> &ConnectionWrapper {
    &self.owned_query_data.db
  }

  fn timezone(&self) -> &chrono_tz::Tz {
    &self.owned_query_data.timezone
  }

  fn user_con_profile(&self) -> Option<&user_con_profiles::Model> {
    self.owned_query_data.user_con_profile.as_ref()
  }
}
