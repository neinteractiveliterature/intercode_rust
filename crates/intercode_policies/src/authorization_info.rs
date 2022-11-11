use intercode_entities::{user_con_profiles, users};
use memo_map::MemoMap;
use oxide_auth::endpoint::Scope;
use sea_orm::{DatabaseConnection, DbErr};
use std::{fmt::Debug, sync::Arc};

use crate::permissions_loading::{
  load_all_permissions_in_convention_with_model_type_and_id, UserPermissionsMap,
};

// #[derive(Debug)]
// pub enum OnceMapError<E: Debug> {
//   PoisonError,
//   WrappedError(E),
// }

// impl<E: Debug> Display for OnceMapError<E> {
//   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//     f.write_fmt(format_args!("{:?}", self))
//   }
// }

// impl<E: Debug> Error for OnceMapError<E> {}

// impl<'a, Key: Debug, Value: Debug, E: Debug>
//   From<PoisonError<MutexGuard<'a, HashMap<Key, OnceBox<Value>>>>> for OnceMapError<E>
// {
//   fn from(_err: PoisonError<MutexGuard<'a, HashMap<Key, OnceBox<Value>>>>) -> Self {
//     OnceMapError::PoisonError
//   }
// }

// pub type OnceMapResult<T, E = Infallible> = Result<T, OnceMapError<E>>;

// #[derive(Debug, Clone, Default)]
// struct OnceMap<Key: Eq + Hash, Value> {
//   storage: Arc<Mutex<HashMap<Key, OnceBox<Value>>>>,
// }

// impl<Key: Eq + Hash + Debug, Value: Debug> OnceMap<Key, Value> {
//   fn get(&self, key: &Key) -> MappedMutexGuard<Option<&Value>> {
//     let lock = self.storage.lock();
//     MutexGuard::map(lock, |storage| &mut storage.get(key).and_then(|v| v.get()))
//   }

//   fn get_or_init<'a, F>(&'a self, key: Key, f: F) -> MappedMutexGuard<&Value>
//   where
//     F: FnOnce() -> Box<Value>,
//   {
//     let lock = self.storage.lock();
//     MutexGuard::map(lock, |storage| {
//       let entry = storage.entry(key);
//       &mut entry.or_default().get_or_init(f)
//     })
//   }

//   async fn get_or_init_async<F, Fut>(&self, key: Key, f: F) -> MappedMutexGuard<&Value>
//   where
//     F: FnOnce() -> Fut,
//     Fut: Future<Output = Box<Value>>,
//   {
//     self.get_or_init(key, || {
//       ::tokio::task::block_in_place(|| ::tokio::runtime::Handle::current().block_on(f()))
//     })
//   }

//   fn get_or_try_init<'a, F, E: 'a>(
//     &'a self,
//     key: Key,
//     f: F,
//   ) -> MappedMutexGuard<Result<&'a Value, E>>
//   where
//     F: FnOnce() -> Result<Box<Value>, E>,
//   {
//     let lock = self.storage.lock();
//     MutexGuard::map(lock, |storage| {
//       let entry = storage.entry(key);
//       &mut entry.or_default().get_or_try_init(f)
//     })
//   }

//   async fn get_or_try_init_async<'a, F, Fut, E: 'a>(
//     &'a self,
//     key: Key,
//     f: F,
//   ) -> MappedMutexGuard<Result<&Value, E>>
//   where
//     F: FnOnce() -> Fut,
//     Fut: Future<Output = Result<Box<Value>, E>>,
//   {
//     self.get_or_try_init(key, || {
//       ::tokio::task::block_in_place(|| ::tokio::runtime::Handle::current().block_on(f()))
//     })
//   }
// }

#[derive(Clone, Debug)]
pub struct AuthorizationInfo {
  pub db: Arc<DatabaseConnection>,
  pub user: Arc<Option<users::Model>>,
  pub oauth_scope: Option<Scope>,
  pub assumed_identity_from_profile: Arc<Option<user_con_profiles::Model>>,
  all_model_permissions_by_convention: MemoMap<i64, UserPermissionsMap>,
}

impl AuthorizationInfo {
  pub fn new(
    db: Arc<DatabaseConnection>,
    user: Arc<Option<users::Model>>,
    oauth_scope: Option<Scope>,
    assumed_identity_from_profile: Arc<Option<user_con_profiles::Model>>,
  ) -> Self {
    Self {
      db,
      user,
      oauth_scope,
      assumed_identity_from_profile,
      all_model_permissions_by_convention: Default::default(),
    }
  }

  pub async fn all_model_permissions_in_convention(
    &self,
    convention_id: i64,
  ) -> Result<&UserPermissionsMap, DbErr> {
    self
      .all_model_permissions_by_convention
      .get_or_try_insert(&convention_id, || {
        let user_id = self.user.as_ref().as_ref().map(|user| user.id);
        ::tokio::task::block_in_place(|| {
          ::tokio::runtime::Handle::current().block_on(
            load_all_permissions_in_convention_with_model_type_and_id(
              &self.db,
              convention_id,
              user_id,
            ),
          )
        })
      })
  }

  pub fn has_scope(&self, scope: &str) -> bool {
    if let Some(my_scope) = &self.oauth_scope {
      my_scope >= &scope.parse::<Scope>().unwrap()
    } else {
      // If there is no OAuth scope, we're a cookied user and therefore scopes don't apply
      true
    }
  }

  pub fn site_admin(&self) -> bool {
    self
      .user
      .as_ref()
      .as_ref()
      .and_then(|u| u.site_admin)
      .unwrap_or(false)
  }

  pub fn site_admin_read(&self) -> bool {
    self.site_admin() && self.has_scope("read_conventions")
  }

  pub fn site_admin_manage(&self) -> bool {
    self.site_admin() && self.has_scope("manage_conventions")
  }
}
