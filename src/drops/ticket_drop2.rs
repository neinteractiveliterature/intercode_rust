use super::{drop_context::DropContext, TicketTypeDrop};
use intercode_entities::tickets;
use seawater::liquid_drop_impl;
use seawater::{belongs_to_related, model_backed_drop, DropError};

#[derive(Debug, Clone)]
pub struct TicketDrop {
  model: tickets::Model,
  #[allow(dead_code)]
  context: DropContext,
}

impl ::seawater::ModelBackedDrop for TicketDrop {
  type Model = tickets::Model;
  fn new(model: tickets::Model, context: DropContext) -> Self {
    TicketDrop { model, context }
  }
  fn get_model(&self) -> &tickets::Model {
    &self.model
  }
}
pub struct TicketDropCache {
  pub id: once_cell::race::OnceBox<::seawater::DropResult<i64>>,
  pub allows_event_signups:
    once_cell::race::OnceBox<::seawater::DropResult<::seawater::ResultValueView<bool, DropError>>>,
  pub ticket_type: once_cell::race::OnceBox<
    ::seawater::DropResult<::seawater::ResultValueView<TicketTypeDrop, ::seawater::DropError>>,
  >,
}
impl TicketDropCache {
  pub fn set_id(
    &self,
    value: ::seawater::DropResult<i64>,
  ) -> Result<(), Box<::seawater::DropResult<i64>>> {
    self.id.set(Box::new(value))
  }
  pub fn get_or_init_id<F>(&self, f: F) -> &::seawater::DropResult<i64>
  where
    F: FnOnce() -> Box<::seawater::DropResult<i64>>,
  {
    self.id.get_or_init(f)
  }
  pub fn set_allows_event_signups(
    &self,
    value: ::seawater::DropResult<::seawater::ResultValueView<bool, DropError>>,
  ) -> Result<(), Box<::seawater::DropResult<::seawater::ResultValueView<bool, DropError>>>> {
    self.allows_event_signups.set(Box::new(value))
  }
  pub fn get_or_init_allows_event_signups<F>(
    &self,
    f: F,
  ) -> &::seawater::DropResult<::seawater::ResultValueView<bool, DropError>>
  where
    F: FnOnce() -> Box<::seawater::DropResult<::seawater::ResultValueView<bool, DropError>>>,
  {
    self.allows_event_signups.get_or_init(f)
  }
  pub fn set_ticket_type(
    &self,
    value: ::seawater::DropResult<
      ::seawater::ResultValueView<TicketTypeDrop, ::seawater::DropError>,
    >,
  ) -> Result<
    (),
    Box<::seawater::DropResult<::seawater::ResultValueView<TicketTypeDrop, ::seawater::DropError>>>,
  > {
    self.ticket_type.set(Box::new(value))
  }
  pub fn get_or_init_ticket_type<F>(
    &self,
    f: F,
  ) -> &::seawater::DropResult<::seawater::ResultValueView<TicketTypeDrop, ::seawater::DropError>>
  where
    F: FnOnce() -> Box<
      ::seawater::DropResult<::seawater::ResultValueView<TicketTypeDrop, ::seawater::DropError>>,
    >,
  {
    self.ticket_type.get_or_init(f)
  }
}
impl ::std::fmt::Debug for TicketDropCache {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    f.debug_struct("TicketDropCache")
      .field("id", &self.id.get())
      .field("allows_event_signups", &self.allows_event_signups.get())
      .field("ticket_type", &self.ticket_type.get())
      .finish()
  }
}
impl Default for TicketDropCache {
  fn default() -> Self {
    Self {
      id: ::once_cell::race::OnceBox::new(),
      allows_event_signups: ::once_cell::race::OnceBox::new(),
      ticket_type: ::once_cell::race::OnceBox::new(),
    }
  }
}
impl ::seawater::LiquidDropCache for TicketDropCache {
  fn new() -> Self {
    Self::default()
  }
}
impl TicketDrop {
  fn get_ticket_type_once_cell(
    cache: &<Self as ::seawater::LiquidDrop>::Cache,
  ) -> &::once_cell::race::OnceBox<::seawater::DropResult<TicketTypeDrop>> {
    &cache.ticket_type
  }
  pub fn ticket_type_preloader(
    context: <Self as ::seawater::LiquidDrop>::Context,
  ) -> ::seawater::preloaders::EntityRelationPreloader<
    ::seawater::DropEntity<Self>,
    ::seawater::DropEntity<TicketTypeDrop>,
    ::seawater::DropPrimaryKey<Self>,
    Self,
    TicketTypeDrop,
    TicketTypeDrop,
    <Self as ::seawater::LiquidDrop>::Context,
  > {
    use ::seawater::LiquidDrop;
    use ::seawater::ModelBackedDrop;
    ::seawater::preloaders::EntityRelationPreloader::<
                ::seawater::DropEntity<Self>,
                ::seawater::DropEntity<TicketTypeDrop>,
                ::seawater::DropPrimaryKey<Self>,
                Self,
                TicketTypeDrop,
                TicketTypeDrop,
                <Self as ::seawater::LiquidDrop>::Context,
            >::new(
                <<<Self as ::seawater::ModelBackedDrop>::Model as ::sea_orm::ModelTrait>::Entity as ::sea_orm::EntityTrait>::PrimaryKey::Id,
                context,
                |
                    result: Option<
                        ::seawater::loaders::EntityRelationLoaderResult<
                            <<Self as ::seawater::ModelBackedDrop>::Model as ::sea_orm::ModelTrait>::Entity,
                            <<TicketTypeDrop as ::seawater::ModelBackedDrop>::Model as ::sea_orm::ModelTrait>::Entity,
                        >,
                    >,
                    from_drop: ::seawater::DropRef<
                        Self,
                        <Self as ::seawater::LiquidDrop>::ID,
                    >|
                {
                    result
                        .map(|result| {
                            Ok(
                                result
                                    .models
                                    .into_iter()
                                    .map(|model| <TicketTypeDrop>::new(
                                        model,
                                        from_drop.context.clone(),
                                    ))
                                    .collect(),
                            )
                        })
                        .unwrap_or_else(|| Ok(::alloc::vec::Vec::new()))
                },
                |
                    store: &::seawater::DropStore<::seawater::DropStoreID<Self>>,
                    drops: Vec<
                        ::seawater::DropRef<
                            TicketTypeDrop,
                            ::seawater::DropStoreID<TicketTypeDrop>,
                        >,
                    >|
                {
                    if drops.len() == 1 {
                        Ok(drops[0].into())
                    } else {
                        Err(
                            ::seawater::DropError::ExpectedEntityNotFound({
                                let res = ::alloc::fmt::format(
                                    format_args!(
                                        "Expected one {0}, but there are {1}",
                                        ::seawater::pretty_type_name::< TicketTypeDrop > (), drops
                                        .len()
                                    ),
                                );
                                res
                            }),
                        )
                    }
                },
                Self::get_ticket_type_once_cell,
                None,
            )
  }
        pub fn preload_ticket_type<'a>(
            context: <Self as ::seawater::LiquidDrop>::Context,
            drops: &'a [::seawater::DropRef<
                Self,
                <<Self as ::seawater::LiquidDrop>::Context as ::seawater::Context>::StoreID,
            >],
        ) -> ::futures::future::BoxFuture<
            'a,
            Result<
                ::seawater::preloaders::PreloaderResult<
                    <<<<Self as ::seawater::ModelBackedDrop>::Model as ::sea_orm::ModelTrait>::Entity as ::sea_orm::EntityTrait>::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType,
                    TicketTypeDrop,
                >,
                ::seawater::DropError,
            >,
  >{
    use ::futures::FutureExt;
    async move {
      use ::seawater::preloaders::Preloader;
      use ::seawater::Context;
      use ::tracing::log::info;
      {
        let lvl = ::log::Level::Info;
        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
          ::log::__private_api_log(
            format_args!(
              "{0}.{1}: eager-loading {2} {3}",
              ::seawater::pretty_type_name::<Self>(),
              "preload_ticket_type",
              drops.len(),
              ::seawater::pretty_type_name::<TicketTypeDrop>()
            ),
            lvl,
            &(
              "intercode_rust::drops::ticket_drop",
              "intercode_rust::drops::ticket_drop",
              "src/drops/ticket_drop.rs",
              9u32,
            ),
            ::log::__private_api::Option::None,
          );
        }
      };
      let preloader = Self::ticket_type_preloader(context.clone());
      let preloader_result = preloader.preload(context.db(), drops).await?;
      let preloaded_drops = preloader_result
        .all_values_unwrapped()
        .cloned()
        .collect::<Vec<_>>();
      Ok(preloader_result)
    }
    .boxed()
  }
  pub async fn caching_id(&self) -> ::seawater::DropResult<i64> {
    use ::seawater::{Context, DropStore, LiquidDrop};
    let cache = self.with_drop_store(|store| store.get_drop_cache::<Self>(self.id()));
    cache.get_or_init_id(|| Box::new(self.id().into())).clone()
  }
  pub async fn uncached_allows_event_signups(&self) -> Result<bool, DropError> {
    let ticket_type_result = self.ticket_type().await;
    let ticket_type = ticket_type_result.get_inner();
    Ok(**ticket_type.allows_event_signups().await.get_inner())
  }
  pub async fn allows_event_signups(
    &self,
  ) -> ::seawater::DropResult<::seawater::ResultValueView<bool, DropError>> {
    use ::seawater::{Context, DropStore, LiquidDrop};
    let cache = self.with_drop_store(|store| store.get_drop_cache::<Self>(self.id()));
    cache
      .get_or_init_allows_event_signups(|| {
        Box::<::seawater::DropResult<::seawater::ResultValueView<bool, DropError>>>::new(
          ::tokio::task::block_in_place(|| {
            ::tokio::runtime::Handle::current()
              .block_on(async move { self.uncached_allows_event_signups().await.into() })
          }),
        )
      })
      .clone()
  }
  pub async fn uncached_ticket_type(&self) -> Result<TicketTypeDrop, ::seawater::DropError> {
    use ::seawater::preloaders::Preloader;
    use ::seawater::Context;
    use ::seawater::LiquidDrop;
    use ::tracing::log::info;
    {
      let lvl = ::log::Level::Info;
      if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
        ::log::__private_api_log(
          format_args!(
            "{0}.{1}: lazy-loading 1 {2}",
            ::seawater::pretty_type_name::<Self>(),
            "ticket_type",
            ::seawater::pretty_type_name::<TicketTypeDrop>()
          ),
          lvl,
          &(
            "intercode_rust::drops::ticket_drop",
            "intercode_rust::drops::ticket_drop",
            "src/drops/ticket_drop.rs",
            9u32,
          ),
          ::log::__private_api::Option::None,
        );
      }
    };
    let drop_ref = self.context.with_drop_store(|store| store.store(self))?;
    let drop = Self::ticket_type_preloader(self.context.clone())
      .expect_single(self.context.db(), drop_ref)
      .await?;
    let context = self.context.clone();
    let preloaded_drops = <[_]>::into_vec(
      #[rustc_box]
      ::alloc::boxed::Box::new([drop.clone()]),
    );
    Ok(drop)
  }
  pub async fn ticket_type(
    &self,
  ) -> ::seawater::DropResult<::seawater::ResultValueView<TicketTypeDrop, ::seawater::DropError>>
  {
    use ::seawater::{Context, DropStore, LiquidDrop};
    let cache = self.with_drop_store(|store| store.get_drop_cache::<Self>(self.id()));
    cache
      .get_or_init_ticket_type(|| {
        Box::<
          ::seawater::DropResult<
            ::seawater::ResultValueView<TicketTypeDrop, ::seawater::DropError>,
          >,
        >::new(::tokio::task::block_in_place(|| {
          ::tokio::runtime::Handle::current()
            .block_on(async move { self.uncached_ticket_type().await.into() })
        }))
      })
      .clone()
  }
}
impl ::seawater::LiquidDrop for TicketDrop {
  type Cache = TicketDropCache;
  type ID = i64;
  type Context = DropContext;
  fn id(&self) -> i64 {
    self.model.id
  }
  fn get_context(&self) -> &Self::Context {
    &self.context
  }
}
impl serde::ser::Serialize for TicketDrop {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::ser::Serializer,
  {
    use ::liquid_core::ValueView;
    use ::seawater::LiquidDrop;
    use ::serde::ser::SerializeStruct;
    let mut struct_serializer = serializer.serialize_struct("TicketDrop", 3usize)?;
    let (caching_id, allows_event_signups, ticket_type) = tokio::task::block_in_place(move || {
      tokio::runtime::Handle::current().block_on(async move {
        {
          use ::futures_util::__private as __futures_crate;
          {
            let mut _fut0 = __futures_crate::future::maybe_done(self.caching_id());
            let mut _fut0 = unsafe { __futures_crate::Pin::new_unchecked(&mut _fut0) };
            let mut _fut1 = __futures_crate::future::maybe_done(self.allows_event_signups());
            let mut _fut1 = unsafe { __futures_crate::Pin::new_unchecked(&mut _fut1) };
            let mut _fut2 = __futures_crate::future::maybe_done(self.ticket_type());
            let mut _fut2 = unsafe { __futures_crate::Pin::new_unchecked(&mut _fut2) };
            __futures_crate::future::poll_fn(
              move |__cx: &mut __futures_crate::task::Context<'_>| {
                let mut __all_done = true;
                __all_done &=
                  __futures_crate::future::Future::poll(_fut0.as_mut(), __cx).is_ready();
                __all_done &=
                  __futures_crate::future::Future::poll(_fut1.as_mut(), __cx).is_ready();
                __all_done &=
                  __futures_crate::future::Future::poll(_fut2.as_mut(), __cx).is_ready();
                if __all_done {
                  __futures_crate::task::Poll::Ready((
                    _fut0.as_mut().take_output().unwrap(),
                    _fut1.as_mut().take_output().unwrap(),
                    _fut2.as_mut().take_output().unwrap(),
                  ))
                } else {
                  __futures_crate::task::Poll::Pending
                }
              },
            )
            .await
          }
        }
      })
    });
    struct_serializer.serialize_field("id", &caching_id.to_value())?;
    struct_serializer.serialize_field("allows_event_signups", &allows_event_signups.to_value())?;
    struct_serializer.serialize_field("ticket_type", &ticket_type.to_value())?;
    struct_serializer.end()
  }
}
impl liquid::ValueView for TicketDrop {
  fn as_debug(&self) -> &dyn std::fmt::Debug {
    self as &dyn std::fmt::Debug
  }
  fn render(&self) -> liquid::model::DisplayCow<'_> {
    liquid::model::DisplayCow::Owned(Box::new("TicketDrop"))
  }
  fn source(&self) -> liquid::model::DisplayCow<'_> {
    liquid::model::DisplayCow::Owned(Box::new("TicketDrop"))
  }
  fn type_name(&self) -> &'static str {
    "TicketDrop"
  }
  fn query_state(&self, state: liquid::model::State) -> bool {
    match state {
      liquid::model::State::Truthy => true,
      liquid::model::State::DefaultValue => false,
      liquid::model::State::Empty => false,
      liquid::model::State::Blank => false,
    }
  }
  fn to_kstr(&self) -> liquid::model::KStringCow<'_> {
    "TicketDrop".to_kstr()
  }
  fn to_value(&self) -> liquid_core::Value {
    liquid::model::Value::Object(liquid::model::Object::from_iter(
      self
        .as_object()
        .unwrap()
        .iter()
        .filter(|(key, value)| match key.as_str() {
          "id" | "allows_event_signups" => true,
          _ => false,
        })
        .map(|(key, value)| (key.into(), value.to_value())),
    ))
  }
  fn as_object(&self) -> Option<&dyn ::liquid::model::ObjectView> {
    Some(self)
  }
}
impl liquid::ObjectView for TicketDrop {
  fn as_value(&self) -> &dyn liquid::ValueView {
    self as &dyn liquid::ValueView
  }
  fn size(&self) -> i64 {
    usize::try_into(3usize).unwrap()
  }
  fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = liquid::model::KStringCow<'k>> + 'k> {
    Box::new(
      <[_]>::into_vec(
        #[rustc_box]
        ::alloc::boxed::Box::new(["id", "allows_event_signups", "ticket_type"]),
      )
      .into_iter()
      .map(|s| s.into()),
    )
  }
  fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn liquid::ValueView> + 'k> {
    use ::seawater::LiquidDrop;
    let (caching_id, allows_event_signups, ticket_type) = tokio::task::block_in_place(move || {
      tokio::runtime::Handle::current().block_on(async move {
        {
          use ::futures_util::__private as __futures_crate;
          {
            let mut _fut0 = __futures_crate::future::maybe_done(self.caching_id());
            let mut _fut0 = unsafe { __futures_crate::Pin::new_unchecked(&mut _fut0) };
            let mut _fut1 = __futures_crate::future::maybe_done(self.allows_event_signups());
            let mut _fut1 = unsafe { __futures_crate::Pin::new_unchecked(&mut _fut1) };
            let mut _fut2 = __futures_crate::future::maybe_done(self.ticket_type());
            let mut _fut2 = unsafe { __futures_crate::Pin::new_unchecked(&mut _fut2) };
            __futures_crate::future::poll_fn(
              move |__cx: &mut __futures_crate::task::Context<'_>| {
                let mut __all_done = true;
                __all_done &=
                  __futures_crate::future::Future::poll(_fut0.as_mut(), __cx).is_ready();
                __all_done &=
                  __futures_crate::future::Future::poll(_fut1.as_mut(), __cx).is_ready();
                __all_done &=
                  __futures_crate::future::Future::poll(_fut2.as_mut(), __cx).is_ready();
                if __all_done {
                  __futures_crate::task::Poll::Ready((
                    _fut0.as_mut().take_output().unwrap(),
                    _fut1.as_mut().take_output().unwrap(),
                    _fut2.as_mut().take_output().unwrap(),
                  ))
                } else {
                  __futures_crate::task::Poll::Pending
                }
              },
            )
            .await
          }
        }
      })
    });
    let values: Vec<&dyn liquid::ValueView> = <[_]>::into_vec(
      #[rustc_box]
      ::alloc::boxed::Box::new([
        caching_id.as_value(),
        allows_event_signups.as_value(),
        ticket_type.as_value(),
      ]),
    );
    Box::new(values.into_iter())
  }
  fn iter<'k>(
    &'k self,
  ) -> Box<dyn Iterator<Item = (liquid::model::KStringCow<'k>, &'k dyn liquid::ValueView)> + 'k> {
    use ::seawater::LiquidDrop;
    let (caching_id, allows_event_signups, ticket_type) = tokio::task::block_in_place(move || {
      tokio::runtime::Handle::current().block_on(async move {
        {
          use ::futures_util::__private as __futures_crate;
          {
            let mut _fut0 = __futures_crate::future::maybe_done(self.caching_id());
            let mut _fut0 = unsafe { __futures_crate::Pin::new_unchecked(&mut _fut0) };
            let mut _fut1 = __futures_crate::future::maybe_done(self.allows_event_signups());
            let mut _fut1 = unsafe { __futures_crate::Pin::new_unchecked(&mut _fut1) };
            let mut _fut2 = __futures_crate::future::maybe_done(self.ticket_type());
            let mut _fut2 = unsafe { __futures_crate::Pin::new_unchecked(&mut _fut2) };
            __futures_crate::future::poll_fn(
              move |__cx: &mut __futures_crate::task::Context<'_>| {
                let mut __all_done = true;
                __all_done &=
                  __futures_crate::future::Future::poll(_fut0.as_mut(), __cx).is_ready();
                __all_done &=
                  __futures_crate::future::Future::poll(_fut1.as_mut(), __cx).is_ready();
                __all_done &=
                  __futures_crate::future::Future::poll(_fut2.as_mut(), __cx).is_ready();
                if __all_done {
                  __futures_crate::task::Poll::Ready((
                    _fut0.as_mut().take_output().unwrap(),
                    _fut1.as_mut().take_output().unwrap(),
                    _fut2.as_mut().take_output().unwrap(),
                  ))
                } else {
                  __futures_crate::task::Poll::Pending
                }
              },
            )
            .await
          }
        }
      })
    });
    let pairs: Vec<(&str, &dyn liquid::ValueView)> = <[_]>::into_vec(
      #[rustc_box]
      ::alloc::boxed::Box::new([
        ("id", caching_id.as_value()),
        ("allows_event_signups", allows_event_signups.as_value()),
        ("ticket_type", ticket_type.as_value()),
      ]),
    );
    Box::new(pairs.into_iter().map(|(key, value)| (key.into(), value)))
  }
  fn contains_key(&self, index: &str) -> bool {
    match index {
      "id" | "allows_event_signups" | "ticket_type" => true,
      _ => false,
    }
  }
  fn get<'s>(&'s self, index: &str) -> Option<&'s dyn liquid::ValueView> {
    use ::seawater::LiquidDrop;
    tokio::task::block_in_place(move || {
      tokio::runtime::Handle::current().block_on(async move {
        match index {
          "id" => Some(self.caching_id().await.as_value()),
          "allows_event_signups" => Some(self.allows_event_signups().await.as_value()),
          "ticket_type" => Some(self.ticket_type().await.as_value()),
          _ => None,
        }
      })
    })
  }
}
impl From<TicketDrop> for ::seawater::DropResult<TicketDrop> {
  fn from(drop: TicketDrop) -> Self {
    ::seawater::DropResult::new(drop.clone())
  }
}
impl ::seawater::IntoDropResult for TicketDrop {}
impl ::seawater::DropResultTrait<TicketDrop> for TicketDrop {
  fn get_inner<'a>(&'a self) -> Box<dyn ::std::ops::Deref<Target = TicketDrop> + 'a> {
    Box::new(self)
  }
}
