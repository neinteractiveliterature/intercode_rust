mod room_drop {
    use intercode_entities::rooms;
    use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
    use seawater::model_backed_drop;
    use super::drop_context::DropContext;
    pub struct RoomDrop {
        model: rooms::Model,
        #[allow(dead_code)]
        context: DropContext,
        pub drop_cache: ::std::sync::Arc<RoomDropCache>,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for RoomDrop {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "RoomDrop",
                "model",
                &&self.model,
                "context",
                &&self.context,
                "drop_cache",
                &&self.drop_cache,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for RoomDrop {
        #[inline]
        fn clone(&self) -> RoomDrop {
            RoomDrop {
                model: ::core::clone::Clone::clone(&self.model),
                context: ::core::clone::Clone::clone(&self.context),
                drop_cache: ::core::clone::Clone::clone(&self.drop_cache),
            }
        }
    }
    impl ::seawater::ContextContainer for RoomDrop {
        type Context = DropContext;
    }
    impl ::seawater::ModelBackedDrop for RoomDrop {
        type Model = rooms::Model;
        fn new(model: rooms::Model, context: DropContext) -> Self {
            RoomDrop {
                model,
                context,
                drop_cache: ::std::default::Default::default(),
            }
        }
        fn get_model(&self) -> &rooms::Model {
            &self.model
        }
    }
    pub struct RoomDropCache {
        id: once_cell::race::OnceBox<::lazy_liquid_value_view::DropResult<i64>>,
        name: once_cell::race::OnceBox<::lazy_liquid_value_view::DropResult<String>>,
    }
    impl RoomDropCache {
        pub fn set_name(
            &self,
            value: ::lazy_liquid_value_view::DropResult<String>,
        ) -> Result<(), Box<::lazy_liquid_value_view::DropResult<String>>> {
            self.name.set(Box::new(value))
        }
    }
    impl ::std::fmt::Debug for RoomDropCache {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            f.debug_struct("RoomDropCache")
                .field("id", &self.caching_id.get())
                .field("name", &self.name.get())
                .finish()
        }
    }
    impl Default for RoomDropCache {
        fn default() -> Self {
            Self {
                id: ::once_cell::race::OnceBox::new(),
                name: ::once_cell::race::OnceBox::new(),
            }
        }
    }
    impl RoomDrop {
        pub async fn caching_id(&self) -> &::lazy_liquid_value_view::DropResult<i64> {
            use ::lazy_liquid_value_view::LiquidDropWithID;
            self.drop_cache
                .id
                .get_or_init(|| Box::new(self.id().into()))
        }
        fn uncached_name(&self) -> Option<&String> {
            self.model.name.as_ref()
        }
        pub async fn name(&self) -> &::lazy_liquid_value_view::DropResult<String> {
            use ::lazy_liquid_value_view::LiquidDropWithID;
            self.drop_cache
                .name
                .get_or_init(|| Box::new(self.uncached_name().into()))
        }
        pub fn extend(
            &self,
            extensions: liquid::model::Object,
        ) -> ::lazy_liquid_value_view::ExtendedDropResult<RoomDrop> {
            ::lazy_liquid_value_view::ExtendedDropResult {
                drop_result: self.into(),
                extensions,
            }
        }
    }
    impl ::lazy_liquid_value_view::LiquidDrop for RoomDrop {
        type Cache = RoomDropCache;
        fn get_cache(&self) -> &RoomDropCache {
            &self.drop_cache
        }
    }
    impl ::lazy_liquid_value_view::LiquidDropWithID for RoomDrop {
        type ID = i64;
        fn id(&self) -> i64 {
            self.model.id
        }
    }
    impl serde::ser::Serialize for RoomDrop {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::ser::Serializer,
        {
            use ::serde::ser::SerializeStruct;
            use ::liquid_core::ValueView;
            use ::lazy_liquid_value_view::LiquidDropWithID;
            let mut struct_serializer = serializer.serialize_struct("RoomDrop", 2usize)?;
            let (caching_id, name) = tokio::task::block_in_place(move || {
                tokio::runtime::Handle::current().block_on(async move {
                    {
                        use ::futures_util::__private as __futures_crate;
                        {
                            let mut _fut0 = __futures_crate::future::maybe_done(self.caching_id());
                            let mut _fut1 = __futures_crate::future::maybe_done(self.name());
                            __futures_crate::future::poll_fn(
                                move |__cx: &mut __futures_crate::task::Context<'_>| {
                                    let mut __all_done = true;
                                    __all_done &= __futures_crate::future::Future::poll(
                                        unsafe { __futures_crate::Pin::new_unchecked(&mut _fut0) },
                                        __cx,
                                    )
                                    .is_ready();
                                    __all_done &= __futures_crate::future::Future::poll(
                                        unsafe { __futures_crate::Pin::new_unchecked(&mut _fut1) },
                                        __cx,
                                    )
                                    .is_ready();
                                    if __all_done {
                                        __futures_crate::task::Poll::Ready((
                                            unsafe {
                                                __futures_crate::Pin::new_unchecked(&mut _fut0)
                                            }
                                            .take_output()
                                            .unwrap(),
                                            unsafe {
                                                __futures_crate::Pin::new_unchecked(&mut _fut1)
                                            }
                                            .take_output()
                                            .unwrap(),
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
            struct_serializer.serialize_field("name", &name.to_value())?;
            struct_serializer.end()
        }
    }
    impl liquid::ValueView for RoomDrop {
        fn as_debug(&self) -> &dyn std::fmt::Debug {
            self as &dyn std::fmt::Debug
        }
        fn render(&self) -> liquid::model::DisplayCow<'_> {
            liquid::model::DisplayCow::Owned(Box::new("RoomDrop"))
        }
        fn source(&self) -> liquid::model::DisplayCow<'_> {
            liquid::model::DisplayCow::Owned(Box::new("RoomDrop"))
        }
        fn type_name(&self) -> &'static str {
            "RoomDrop"
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
            "RoomDrop".to_kstr()
        }
        fn to_value(&self) -> liquid_core::Value {
            liquid::model::Value::Object(liquid::model::Object::from_iter(
                self.as_object()
                    .unwrap()
                    .iter()
                    .filter(|(key, value)| match key.as_str() {
                        "id" | "name" => true,
                        _ => false,
                    })
                    .map(|(key, value)| (key.into(), value.to_value())),
            ))
        }
        fn as_object(&self) -> Option<&dyn ::liquid::model::ObjectView> {
            Some(self)
        }
    }
    impl liquid::ObjectView for RoomDrop {
        fn as_value(&self) -> &dyn liquid::ValueView {
            self as &dyn liquid::ValueView
        }
        fn size(&self) -> i64 {
            usize::try_into(2usize).unwrap()
        }
        fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = liquid::model::KStringCow<'k>> + 'k> {
            Box::new(
                <[_]>::into_vec(
                    #[rustc_box]
                    ::alloc::boxed::Box::new(["id", "name"]),
                )
                .into_iter()
                .map(|s| s.into()),
            )
        }
        fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn liquid::ValueView> + 'k> {
            use ::lazy_liquid_value_view::LiquidDropWithID;
            let (caching_id, name) = tokio::task::block_in_place(move || {
                tokio::runtime::Handle::current().block_on(async move {
                    {
                        use ::futures_util::__private as __futures_crate;
                        {
                            let mut _fut0 = __futures_crate::future::maybe_done(self.caching_id());
                            let mut _fut1 = __futures_crate::future::maybe_done(self.name());
                            __futures_crate::future::poll_fn(
                                move |__cx: &mut __futures_crate::task::Context<'_>| {
                                    let mut __all_done = true;
                                    __all_done &= __futures_crate::future::Future::poll(
                                        unsafe { __futures_crate::Pin::new_unchecked(&mut _fut0) },
                                        __cx,
                                    )
                                    .is_ready();
                                    __all_done &= __futures_crate::future::Future::poll(
                                        unsafe { __futures_crate::Pin::new_unchecked(&mut _fut1) },
                                        __cx,
                                    )
                                    .is_ready();
                                    if __all_done {
                                        __futures_crate::task::Poll::Ready((
                                            unsafe {
                                                __futures_crate::Pin::new_unchecked(&mut _fut0)
                                            }
                                            .take_output()
                                            .unwrap(),
                                            unsafe {
                                                __futures_crate::Pin::new_unchecked(&mut _fut1)
                                            }
                                            .take_output()
                                            .unwrap(),
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
                ::alloc::boxed::Box::new([caching_id, name]),
            );
            Box::new(
                values
                    .into_iter()
                    .map(|drop_result| drop_result as &dyn ::liquid::ValueView),
            )
        }
        fn iter<'k>(
            &'k self,
        ) -> Box<dyn Iterator<Item = (liquid::model::KStringCow<'k>, &'k dyn liquid::ValueView)> + 'k>
        {
            use ::lazy_liquid_value_view::LiquidDropWithID;
            let (caching_id, name) = tokio::task::block_in_place(move || {
                tokio::runtime::Handle::current().block_on(async move {
                    {
                        use ::futures_util::__private as __futures_crate;
                        {
                            let mut _fut0 = __futures_crate::future::maybe_done(self.caching_id());
                            let mut _fut1 = __futures_crate::future::maybe_done(self.name());
                            __futures_crate::future::poll_fn(
                                move |__cx: &mut __futures_crate::task::Context<'_>| {
                                    let mut __all_done = true;
                                    __all_done &= __futures_crate::future::Future::poll(
                                        unsafe { __futures_crate::Pin::new_unchecked(&mut _fut0) },
                                        __cx,
                                    )
                                    .is_ready();
                                    __all_done &= __futures_crate::future::Future::poll(
                                        unsafe { __futures_crate::Pin::new_unchecked(&mut _fut1) },
                                        __cx,
                                    )
                                    .is_ready();
                                    if __all_done {
                                        __futures_crate::task::Poll::Ready((
                                            unsafe {
                                                __futures_crate::Pin::new_unchecked(&mut _fut0)
                                            }
                                            .take_output()
                                            .unwrap(),
                                            unsafe {
                                                __futures_crate::Pin::new_unchecked(&mut _fut1)
                                            }
                                            .take_output()
                                            .unwrap(),
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
                ::alloc::boxed::Box::new([("id", caching_id), ("name", name)]),
            );
            Box::new(
                pairs
                    .into_iter()
                    .map(|(key, value)| (key.into(), value as &dyn ::liquid::ValueView)),
            )
        }
        fn contains_key(&self, index: &str) -> bool {
            match index {
                "id" | "name" => true,
                _ => false,
            }
        }
        fn get<'s>(&'s self, index: &str) -> Option<&'s dyn liquid::ValueView> {
            use ::lazy_liquid_value_view::LiquidDropWithID;
            tokio::task::block_in_place(move || {
                tokio::runtime::Handle::current().block_on(async move {
                    match index {
                        "id" => Some(self.caching_id().await as &dyn liquid::ValueView),
                        "name" => Some(self.name().await as &dyn liquid::ValueView),
                        _ => None,
                    }
                })
            })
        }
    }
    impl From<RoomDrop> for ::lazy_liquid_value_view::DropResult<RoomDrop> {
        fn from(drop: RoomDrop) -> Self {
            ::lazy_liquid_value_view::DropResult::new(drop.clone())
        }
    }
}
