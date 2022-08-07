mod user_con_profile_drop {
    use intercode_entities::user_con_profiles;
    use intercode_graphql::loaders::expect::ExpectModels;
    use intercode_graphql::SchemaData;
    use intercode_inflector::IntercodeInflector;
    use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
    use super::{DropError, SignupDrop};
    pub struct UserConProfileDrop<'cache> {
        user_con_profile: user_con_profiles::Model,
        schema_data: SchemaData,
        drop_cache: UserConProfileDropCache<'cache>,
    }
    #[automatically_derived]
    impl<'cache> ::core::fmt::Debug for UserConProfileDrop<'cache> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "UserConProfileDrop",
                "user_con_profile",
                &&self.user_con_profile,
                "schema_data",
                &&self.schema_data,
                "drop_cache",
                &&self.drop_cache,
            )
        }
    }
    #[automatically_derived]
    impl<'cache> ::core::clone::Clone for UserConProfileDrop<'cache> {
        #[inline]
        fn clone(&self) -> UserConProfileDrop<'cache> {
            UserConProfileDrop {
                user_con_profile: ::core::clone::Clone::clone(&self.user_con_profile),
                schema_data: ::core::clone::Clone::clone(&self.schema_data),
                drop_cache: ::core::clone::Clone::clone(&self.drop_cache),
            }
        }
    }
    struct UserConProfileDropCache<'cache> {
        id: tokio::sync::OnceCell<::lazy_liquid_value_view::DropResult<'cache>>,
        first_name: tokio::sync::OnceCell<::lazy_liquid_value_view::DropResult<'cache>>,
        last_name: tokio::sync::OnceCell<::lazy_liquid_value_view::DropResult<'cache>>,
        privileges: tokio::sync::OnceCell<::lazy_liquid_value_view::DropResult<'cache>>,
        signups: tokio::sync::OnceCell<::lazy_liquid_value_view::DropResult<'cache>>,
    }
    #[automatically_derived]
    impl<'cache> ::core::fmt::Debug for UserConProfileDropCache<'cache> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field5_finish(
                f,
                "UserConProfileDropCache",
                "id",
                &&self.id,
                "first_name",
                &&self.first_name,
                "last_name",
                &&self.last_name,
                "privileges",
                &&self.privileges,
                "signups",
                &&self.signups,
            )
        }
    }
    #[automatically_derived]
    impl<'cache> ::core::clone::Clone for UserConProfileDropCache<'cache> {
        #[inline]
        fn clone(&self) -> UserConProfileDropCache<'cache> {
            UserConProfileDropCache {
                id: ::core::clone::Clone::clone(&self.id),
                first_name: ::core::clone::Clone::clone(&self.first_name),
                last_name: ::core::clone::Clone::clone(&self.last_name),
                privileges: ::core::clone::Clone::clone(&self.privileges),
                signups: ::core::clone::Clone::clone(&self.signups),
            }
        }
    }
    #[automatically_derived]
    impl<'cache> ::core::default::Default for UserConProfileDropCache<'cache> {
        #[inline]
        fn default() -> UserConProfileDropCache<'cache> {
            UserConProfileDropCache {
                id: ::core::default::Default::default(),
                first_name: ::core::default::Default::default(),
                last_name: ::core::default::Default::default(),
                privileges: ::core::default::Default::default(),
                signups: ::core::default::Default::default(),
            }
        }
    }
    impl<'cache> UserConProfileDrop<'cache> {
        pub fn new(user_con_profile: user_con_profiles::Model, schema_data: SchemaData) -> Self {
            UserConProfileDrop {
                user_con_profile,
                schema_data,
                drop_cache: Default::default(),
            }
        }
        fn id(&self) -> i64 {
            self.user_con_profile.id
        }
        async fn caching_id(&self) -> &::lazy_liquid_value_view::DropResult<'_> {
            self.drop_cache
                .id
                .get_or_init(|| async move { self.id().into() })
                .await
        }
        fn first_name(&self) -> &str {
            self.user_con_profile.first_name.as_str()
        }
        async fn caching_first_name(&self) -> &::lazy_liquid_value_view::DropResult<'_> {
            self.drop_cache
                .first_name
                .get_or_init(|| async move { self.first_name().into() })
                .await
        }
        fn last_name(&self) -> &str {
            self.user_con_profile.last_name.as_str()
        }
        async fn caching_last_name(&self) -> &::lazy_liquid_value_view::DropResult<'_> {
            self.drop_cache
                .last_name
                .get_or_init(|| async move { self.last_name().into() })
                .await
        }
        async fn privileges(&self) -> Result<Vec<String>, DropError> {
            let result = self
                .schema_data
                .loaders
                .user_con_profile_user
                .load_one(self.user_con_profile.id)
                .await?;
            let user = result.expect_one()?;
            let inflector = IntercodeInflector::new();
            Ok(user
                .privileges()
                .iter()
                .map(|priv_name| inflector.humanize(priv_name))
                .collect::<Vec<_>>())
        }
        async fn caching_privileges(&self) -> &::lazy_liquid_value_view::DropResult<'_> {
            self.drop_cache
                .privileges
                .get_or_init(|| async move { self.privileges().await.into() })
                .await
        }
        async fn signups(&self) -> Result<Vec<SignupDrop<'cache>>, DropError> {
            let result = self
                .schema_data
                .loaders
                .user_con_profile_signups
                .load_one(self.user_con_profile.id)
                .await?;
            let signups = result.expect_models()?;
            Ok(signups
                .iter()
                .map(|signup| SignupDrop::new(signup.clone()))
                .collect::<Vec<_>>())
        }
        async fn caching_signups(&self) -> &::lazy_liquid_value_view::DropResult<'_> {
            self.drop_cache
                .signups
                .get_or_init(|| async move { self.signups().await.into() })
                .await
        }
        pub fn extend(
            &self,
            extensions: liquid::model::Object,
        ) -> ::lazy_liquid_value_view::ExtendedDropResult<'_> {
            ::lazy_liquid_value_view::ExtendedDropResult {
                drop_result: self.into(),
                extensions,
            }
        }
    }
    impl<'cache> serde::ser::Serialize for UserConProfileDrop<'cache> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::ser::Serializer,
        {
            use ::serde::ser::SerializeStruct;
            use ::liquid_core::ValueView;
            let mut struct_serializer =
                serializer.serialize_struct("UserConProfileDrop", 5usize)?;
            let (id, first_name, last_name, privileges, signups) =
                tokio::task::block_in_place(move || {
                    tokio::runtime::Handle::current().block_on(async move {
                        {
                            use ::futures_util::__private as __futures_crate;
                            {
                                let mut _fut0 =
                                    __futures_crate::future::maybe_done(self.caching_id());
                                let mut _fut1 =
                                    __futures_crate::future::maybe_done(self.caching_first_name());
                                let mut _fut2 =
                                    __futures_crate::future::maybe_done(self.caching_last_name());
                                let mut _fut3 =
                                    __futures_crate::future::maybe_done(self.caching_privileges());
                                let mut _fut4 =
                                    __futures_crate::future::maybe_done(self.caching_signups());
                                __futures_crate::future::poll_fn(
                                    move |__cx: &mut __futures_crate::task::Context<'_>| {
                                        let mut __all_done = true;
                                        __all_done &= __futures_crate::future::Future::poll(
                                            unsafe {
                                                __futures_crate::Pin::new_unchecked(&mut _fut0)
                                            },
                                            __cx,
                                        )
                                        .is_ready();
                                        __all_done &= __futures_crate::future::Future::poll(
                                            unsafe {
                                                __futures_crate::Pin::new_unchecked(&mut _fut1)
                                            },
                                            __cx,
                                        )
                                        .is_ready();
                                        __all_done &= __futures_crate::future::Future::poll(
                                            unsafe {
                                                __futures_crate::Pin::new_unchecked(&mut _fut2)
                                            },
                                            __cx,
                                        )
                                        .is_ready();
                                        __all_done &= __futures_crate::future::Future::poll(
                                            unsafe {
                                                __futures_crate::Pin::new_unchecked(&mut _fut3)
                                            },
                                            __cx,
                                        )
                                        .is_ready();
                                        __all_done &= __futures_crate::future::Future::poll(
                                            unsafe {
                                                __futures_crate::Pin::new_unchecked(&mut _fut4)
                                            },
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
                                                unsafe {
                                                    __futures_crate::Pin::new_unchecked(&mut _fut2)
                                                }
                                                .take_output()
                                                .unwrap(),
                                                unsafe {
                                                    __futures_crate::Pin::new_unchecked(&mut _fut3)
                                                }
                                                .take_output()
                                                .unwrap(),
                                                unsafe {
                                                    __futures_crate::Pin::new_unchecked(&mut _fut4)
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
            struct_serializer.serialize_field("id", &id.to_value())?;
            struct_serializer.serialize_field("first_name", &first_name.to_value())?;
            struct_serializer.serialize_field("last_name", &last_name.to_value())?;
            struct_serializer.serialize_field("privileges", &privileges.to_value())?;
            struct_serializer.serialize_field("signups", &signups.to_value())?;
            struct_serializer.end()
        }
    }
    impl<'cache> liquid::ValueView for UserConProfileDrop<'cache> {
        fn as_debug(&self) -> &dyn std::fmt::Debug {
            self as &dyn std::fmt::Debug
        }
        fn render(&self) -> liquid::model::DisplayCow<'_> {
            liquid::model::DisplayCow::Owned(Box::new("UserConProfileDrop"))
        }
        fn source(&self) -> liquid::model::DisplayCow<'_> {
            liquid::model::DisplayCow::Owned(Box::new("UserConProfileDrop"))
        }
        fn type_name(&self) -> &'static str {
            "UserConProfileDrop"
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
            "UserConProfileDrop".to_kstr()
        }
        fn to_value(&self) -> liquid_core::Value {
            {
                ::std::io::_print(::core::fmt::Arguments::new_v1(
                    &["Warning!  to_value called on ", "\n"],
                    &[::core::fmt::ArgumentV1::new_display(&"UserConProfileDrop")],
                ));
            };
            liquid::model::Value::Object(liquid::model::Object::from_iter(
                self.as_object()
                    .unwrap()
                    .iter()
                    .map(|(key, value)| (key.into(), value.to_value())),
            ))
        }
        fn as_object(&self) -> Option<&dyn ::liquid::model::ObjectView> {
            Some(self)
        }
    }
    impl<'cache> liquid::ObjectView for UserConProfileDrop<'cache> {
        fn as_value(&self) -> &dyn liquid::ValueView {
            self as &dyn liquid::ValueView
        }
        fn size(&self) -> i64 {
            usize::try_into(5usize).unwrap()
        }
        fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = liquid::model::KStringCow<'k>> + 'k> {
            Box::new(
                <[_]>::into_vec(
                    #[rustc_box]
                    ::alloc::boxed::Box::new([
                        "id",
                        "first_name",
                        "last_name",
                        "privileges",
                        "signups",
                    ]),
                )
                .into_iter()
                .map(|s| s.into()),
            )
        }
        fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn liquid::ValueView> + 'k> {
            let (id, first_name, last_name, privileges, signups) =
                tokio::task::block_in_place(move || {
                    tokio::runtime::Handle::current().block_on(async move {
                        {
                            use ::futures_util::__private as __futures_crate;
                            {
                                let mut _fut0 =
                                    __futures_crate::future::maybe_done(self.caching_id());
                                let mut _fut1 =
                                    __futures_crate::future::maybe_done(self.caching_first_name());
                                let mut _fut2 =
                                    __futures_crate::future::maybe_done(self.caching_last_name());
                                let mut _fut3 =
                                    __futures_crate::future::maybe_done(self.caching_privileges());
                                let mut _fut4 =
                                    __futures_crate::future::maybe_done(self.caching_signups());
                                __futures_crate::future::poll_fn(
                                    move |__cx: &mut __futures_crate::task::Context<'_>| {
                                        let mut __all_done = true;
                                        __all_done &= __futures_crate::future::Future::poll(
                                            unsafe {
                                                __futures_crate::Pin::new_unchecked(&mut _fut0)
                                            },
                                            __cx,
                                        )
                                        .is_ready();
                                        __all_done &= __futures_crate::future::Future::poll(
                                            unsafe {
                                                __futures_crate::Pin::new_unchecked(&mut _fut1)
                                            },
                                            __cx,
                                        )
                                        .is_ready();
                                        __all_done &= __futures_crate::future::Future::poll(
                                            unsafe {
                                                __futures_crate::Pin::new_unchecked(&mut _fut2)
                                            },
                                            __cx,
                                        )
                                        .is_ready();
                                        __all_done &= __futures_crate::future::Future::poll(
                                            unsafe {
                                                __futures_crate::Pin::new_unchecked(&mut _fut3)
                                            },
                                            __cx,
                                        )
                                        .is_ready();
                                        __all_done &= __futures_crate::future::Future::poll(
                                            unsafe {
                                                __futures_crate::Pin::new_unchecked(&mut _fut4)
                                            },
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
                                                unsafe {
                                                    __futures_crate::Pin::new_unchecked(&mut _fut2)
                                                }
                                                .take_output()
                                                .unwrap(),
                                                unsafe {
                                                    __futures_crate::Pin::new_unchecked(&mut _fut3)
                                                }
                                                .take_output()
                                                .unwrap(),
                                                unsafe {
                                                    __futures_crate::Pin::new_unchecked(&mut _fut4)
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
            let values = <[_]>::into_vec(
                #[rustc_box]
                ::alloc::boxed::Box::new([id, first_name, last_name, privileges, signups]),
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
            let (id, first_name, last_name, privileges, signups) =
                tokio::task::block_in_place(move || {
                    tokio::runtime::Handle::current().block_on(async move {
                        {
                            use ::futures_util::__private as __futures_crate;
                            {
                                let mut _fut0 =
                                    __futures_crate::future::maybe_done(self.caching_id());
                                let mut _fut1 =
                                    __futures_crate::future::maybe_done(self.caching_first_name());
                                let mut _fut2 =
                                    __futures_crate::future::maybe_done(self.caching_last_name());
                                let mut _fut3 =
                                    __futures_crate::future::maybe_done(self.caching_privileges());
                                let mut _fut4 =
                                    __futures_crate::future::maybe_done(self.caching_signups());
                                __futures_crate::future::poll_fn(
                                    move |__cx: &mut __futures_crate::task::Context<'_>| {
                                        let mut __all_done = true;
                                        __all_done &= __futures_crate::future::Future::poll(
                                            unsafe {
                                                __futures_crate::Pin::new_unchecked(&mut _fut0)
                                            },
                                            __cx,
                                        )
                                        .is_ready();
                                        __all_done &= __futures_crate::future::Future::poll(
                                            unsafe {
                                                __futures_crate::Pin::new_unchecked(&mut _fut1)
                                            },
                                            __cx,
                                        )
                                        .is_ready();
                                        __all_done &= __futures_crate::future::Future::poll(
                                            unsafe {
                                                __futures_crate::Pin::new_unchecked(&mut _fut2)
                                            },
                                            __cx,
                                        )
                                        .is_ready();
                                        __all_done &= __futures_crate::future::Future::poll(
                                            unsafe {
                                                __futures_crate::Pin::new_unchecked(&mut _fut3)
                                            },
                                            __cx,
                                        )
                                        .is_ready();
                                        __all_done &= __futures_crate::future::Future::poll(
                                            unsafe {
                                                __futures_crate::Pin::new_unchecked(&mut _fut4)
                                            },
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
                                                unsafe {
                                                    __futures_crate::Pin::new_unchecked(&mut _fut2)
                                                }
                                                .take_output()
                                                .unwrap(),
                                                unsafe {
                                                    __futures_crate::Pin::new_unchecked(&mut _fut3)
                                                }
                                                .take_output()
                                                .unwrap(),
                                                unsafe {
                                                    __futures_crate::Pin::new_unchecked(&mut _fut4)
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
            let pairs = <[_]>::into_vec(
                #[rustc_box]
                ::alloc::boxed::Box::new([
                    ("id", id),
                    ("first_name", first_name),
                    ("last_name", last_name),
                    ("privileges", privileges),
                    ("signups", signups),
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
                "id" | "first_name" | "last_name" | "privileges" | "signups" => true,
                _ => false,
            }
        }
        fn get<'s>(&'s self, index: &str) -> Option<&'s dyn liquid::ValueView> {
            tokio::task::block_in_place(move || {
                tokio::runtime::Handle::current().block_on(async move {
                    match index {
                        "id" => Some(self.caching_id().await as &dyn liquid::ValueView),
                        "first_name" => {
                            Some(self.caching_first_name().await as &dyn liquid::ValueView)
                        }
                        "last_name" => {
                            Some(self.caching_last_name().await as &dyn liquid::ValueView)
                        }
                        "privileges" => {
                            Some(self.caching_privileges().await as &dyn liquid::ValueView)
                        }
                        "signups" => Some(self.caching_signups().await as &dyn liquid::ValueView),
                        _ => None,
                    }
                })
            })
        }
    }
    impl<'a, 'cache: 'a> From<UserConProfileDrop<'cache>> for ::lazy_liquid_value_view::DropResult<'a> {
        fn from(drop: UserConProfileDrop<'cache>) -> Self {
            ::lazy_liquid_value_view::DropResult::new(drop.clone())
        }
    }
    impl<'a, 'cache: 'a> From<&'a UserConProfileDrop<'cache>>
        for ::lazy_liquid_value_view::DropResult<'a>
    {
        fn from(drop: &'a UserConProfileDrop<'cache>) -> Self {
            ::lazy_liquid_value_view::DropResult::new(drop.clone())
        }
    }
}
