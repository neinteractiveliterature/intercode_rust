mod event_category_drop {
    use super::drop_context::DropContext;
    use intercode_entities::event_categories;
    use seawater::liquid_drop_impl;
    use seawater::model_backed_drop;
    pub struct EventCategoryDrop {
        model: event_categories::Model,
        #[allow(dead_code)]
        context: DropContext,
        pub drop_cache: ::std::sync::Arc<EventCategoryDropCache>,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for EventCategoryDrop {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "EventCategoryDrop",
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
    impl ::core::clone::Clone for EventCategoryDrop {
        #[inline]
        fn clone(&self) -> EventCategoryDrop {
            EventCategoryDrop {
                model: ::core::clone::Clone::clone(&self.model),
                context: ::core::clone::Clone::clone(&self.context),
                drop_cache: ::core::clone::Clone::clone(&self.drop_cache),
            }
        }
    }
    impl ::seawater::ContextContainer for EventCategoryDrop {
        type Context = DropContext;
    }
    impl ::seawater::ModelBackedDrop for EventCategoryDrop {
        type Model = event_categories::Model;
        fn new(model: event_categories::Model, context: DropContext) -> Self {
            EventCategoryDrop {
                model,
                context,
                drop_cache: ::std::default::Default::default(),
            }
        }
        fn get_model(&self) -> &event_categories::Model {
            &self.model
        }
    }
    pub struct EventCategoryDropCache {
        pub id: once_cell::race::OnceBox<::seawater::DropResult<i64>>,
        pub name: once_cell::race::OnceBox<::seawater::DropResult<String>>,
        pub team_member_name: once_cell::race::OnceBox<::seawater::DropResult<String>>,
    }
    impl EventCategoryDropCache {
        pub fn set_name(
            &self,
            value: ::seawater::DropResult<String>,
        ) -> Result<(), Box<::seawater::DropResult<String>>> {
            self.name.set(Box::new(value))
        }
        pub fn set_team_member_name(
            &self,
            value: ::seawater::DropResult<String>,
        ) -> Result<(), Box<::seawater::DropResult<String>>> {
            self.team_member_name.set(Box::new(value))
        }
    }
    impl ::std::fmt::Debug for EventCategoryDropCache {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            f.debug_struct("EventCategoryDropCache")
                .field("id", &self.id.get())
                .field("name", &self.name.get())
                .field("team_member_name", &self.team_member_name.get())
                .finish()
        }
    }
    impl Default for EventCategoryDropCache {
        fn default() -> Self {
            Self {
                id: ::once_cell::race::OnceBox::new(),
                name: ::once_cell::race::OnceBox::new(),
                team_member_name: ::once_cell::race::OnceBox::new(),
            }
        }
    }
    impl EventCategoryDrop {
        pub async fn caching_id(&self) -> &::seawater::DropResult<i64> {
            use ::seawater::LiquidDrop;
            self.drop_cache
                .id
                .get_or_init(|| Box::new(self.id().into()))
        }
        fn uncached_name(&self) -> &str {
            &self.model.name
        }
        pub async fn name(&self) -> &::seawater::DropResult<String> {
            use ::seawater::LiquidDrop;
            self.drop_cache
                .name
                .get_or_init(|| Box::new(self.uncached_name().into()))
        }
        fn uncached_team_member_name(&self) -> &str {
            &self.model.team_member_name
        }
        pub async fn team_member_name(&self) -> &::seawater::DropResult<String> {
            use ::seawater::LiquidDrop;
            self.drop_cache
                .team_member_name
                .get_or_init(|| Box::new(self.uncached_team_member_name().into()))
        }
        pub fn extend(
            &self,
            extensions: liquid::model::Object,
        ) -> ::seawater::ExtendedDropResult<EventCategoryDrop> {
            ::seawater::ExtendedDropResult {
                drop_result: self.into(),
                extensions,
            }
        }
    }
    impl ::seawater::LiquidDrop for EventCategoryDrop {
        type Cache = EventCategoryDropCache;
        type ID = i64;
        fn id(&self) -> i64 {
            self.model.id
        }
    }
    impl serde::ser::Serialize for EventCategoryDrop {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::ser::Serializer,
        {
            use ::serde::ser::SerializeStruct;
            use ::liquid_core::ValueView;
            use ::seawater::LiquidDrop;
            let mut struct_serializer = serializer.serialize_struct("EventCategoryDrop", 3usize)?;
            let (caching_id, name, team_member_name) = tokio::task::block_in_place(move || {
                tokio::runtime::Handle::current().block_on(async move {
                    {
                        use ::futures_util::__private as __futures_crate;
                        {
                            let mut _fut0 = __futures_crate::future::maybe_done(self.caching_id());
                            let mut _fut0 =
                                unsafe { __futures_crate::Pin::new_unchecked(&mut _fut0) };
                            let mut _fut1 = __futures_crate::future::maybe_done(self.name());
                            let mut _fut1 =
                                unsafe { __futures_crate::Pin::new_unchecked(&mut _fut1) };
                            let mut _fut2 =
                                __futures_crate::future::maybe_done(self.team_member_name());
                            let mut _fut2 =
                                unsafe { __futures_crate::Pin::new_unchecked(&mut _fut2) };
                            __futures_crate::future::poll_fn(
                                move |__cx: &mut __futures_crate::task::Context<'_>| {
                                    let mut __all_done = true;
                                    __all_done &=
                                        __futures_crate::future::Future::poll(_fut0.as_mut(), __cx)
                                            .is_ready();
                                    __all_done &=
                                        __futures_crate::future::Future::poll(_fut1.as_mut(), __cx)
                                            .is_ready();
                                    __all_done &=
                                        __futures_crate::future::Future::poll(_fut2.as_mut(), __cx)
                                            .is_ready();
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
            struct_serializer.serialize_field("name", &name.to_value())?;
            struct_serializer.serialize_field("team_member_name", &team_member_name.to_value())?;
            struct_serializer.end()
        }
    }
    impl liquid::ValueView for EventCategoryDrop {
        fn as_debug(&self) -> &dyn std::fmt::Debug {
            self as &dyn std::fmt::Debug
        }
        fn render(&self) -> liquid::model::DisplayCow<'_> {
            liquid::model::DisplayCow::Owned(Box::new("EventCategoryDrop"))
        }
        fn source(&self) -> liquid::model::DisplayCow<'_> {
            liquid::model::DisplayCow::Owned(Box::new("EventCategoryDrop"))
        }
        fn type_name(&self) -> &'static str {
            "EventCategoryDrop"
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
            "EventCategoryDrop".to_kstr()
        }
        fn to_value(&self) -> liquid_core::Value {
            liquid::model::Value::Object(liquid::model::Object::from_iter(
                self.as_object()
                    .unwrap()
                    .iter()
                    .filter(|(key, value)| match key.as_str() {
                        "id" | "name" | "team_member_name" => true,
                        _ => false,
                    })
                    .map(|(key, value)| (key.into(), value.to_value())),
            ))
        }
        fn as_object(&self) -> Option<&dyn ::liquid::model::ObjectView> {
            Some(self)
        }
    }
    impl liquid::ObjectView for EventCategoryDrop {
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
                    ::alloc::boxed::Box::new(["id", "name", "team_member_name"]),
                )
                .into_iter()
                .map(|s| s.into()),
            )
        }
        fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn liquid::ValueView> + 'k> {
            use ::seawater::LiquidDrop;
            let (caching_id, name, team_member_name) = tokio::task::block_in_place(move || {
                tokio::runtime::Handle::current().block_on(async move {
                    {
                        use ::futures_util::__private as __futures_crate;
                        {
                            let mut _fut0 = __futures_crate::future::maybe_done(self.caching_id());
                            let mut _fut0 =
                                unsafe { __futures_crate::Pin::new_unchecked(&mut _fut0) };
                            let mut _fut1 = __futures_crate::future::maybe_done(self.name());
                            let mut _fut1 =
                                unsafe { __futures_crate::Pin::new_unchecked(&mut _fut1) };
                            let mut _fut2 =
                                __futures_crate::future::maybe_done(self.team_member_name());
                            let mut _fut2 =
                                unsafe { __futures_crate::Pin::new_unchecked(&mut _fut2) };
                            __futures_crate::future::poll_fn(
                                move |__cx: &mut __futures_crate::task::Context<'_>| {
                                    let mut __all_done = true;
                                    __all_done &=
                                        __futures_crate::future::Future::poll(_fut0.as_mut(), __cx)
                                            .is_ready();
                                    __all_done &=
                                        __futures_crate::future::Future::poll(_fut1.as_mut(), __cx)
                                            .is_ready();
                                    __all_done &=
                                        __futures_crate::future::Future::poll(_fut2.as_mut(), __cx)
                                            .is_ready();
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
                ::alloc::boxed::Box::new([caching_id, name, team_member_name]),
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
            use ::seawater::LiquidDrop;
            let (caching_id, name, team_member_name) = tokio::task::block_in_place(move || {
                tokio::runtime::Handle::current().block_on(async move {
                    {
                        use ::futures_util::__private as __futures_crate;
                        {
                            let mut _fut0 = __futures_crate::future::maybe_done(self.caching_id());
                            let mut _fut0 =
                                unsafe { __futures_crate::Pin::new_unchecked(&mut _fut0) };
                            let mut _fut1 = __futures_crate::future::maybe_done(self.name());
                            let mut _fut1 =
                                unsafe { __futures_crate::Pin::new_unchecked(&mut _fut1) };
                            let mut _fut2 =
                                __futures_crate::future::maybe_done(self.team_member_name());
                            let mut _fut2 =
                                unsafe { __futures_crate::Pin::new_unchecked(&mut _fut2) };
                            __futures_crate::future::poll_fn(
                                move |__cx: &mut __futures_crate::task::Context<'_>| {
                                    let mut __all_done = true;
                                    __all_done &=
                                        __futures_crate::future::Future::poll(_fut0.as_mut(), __cx)
                                            .is_ready();
                                    __all_done &=
                                        __futures_crate::future::Future::poll(_fut1.as_mut(), __cx)
                                            .is_ready();
                                    __all_done &=
                                        __futures_crate::future::Future::poll(_fut2.as_mut(), __cx)
                                            .is_ready();
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
                    ("id", caching_id),
                    ("name", name),
                    ("team_member_name", team_member_name),
                ]),
            );
            Box::new(
                pairs
                    .into_iter()
                    .map(|(key, value)| (key.into(), value as &dyn ::liquid::ValueView)),
            )
        }
        fn contains_key(&self, index: &str) -> bool {
            match index {
                "id" | "name" | "team_member_name" => true,
                _ => false,
            }
        }
        fn get<'s>(&'s self, index: &str) -> Option<&'s dyn liquid::ValueView> {
            use ::seawater::LiquidDrop;
            tokio::task::block_in_place(move || {
                tokio::runtime::Handle::current().block_on(async move {
                    match index {
                        "id" => Some(self.caching_id().await as &dyn liquid::ValueView),
                        "name" => Some(self.name().await as &dyn liquid::ValueView),
                        "team_member_name" => {
                            Some(self.team_member_name().await as &dyn liquid::ValueView)
                        }
                        _ => None,
                    }
                })
            })
        }
    }
    impl From<EventCategoryDrop> for ::seawater::DropResult<EventCategoryDrop> {
        fn from(drop: EventCategoryDrop) -> Self {
            ::seawater::DropResult::new(drop.clone())
        }
    }
}
