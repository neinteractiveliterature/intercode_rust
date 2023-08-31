use std::{fmt::Debug, future::Future};

use async_graphql::{ObjectType, Request, Schema, SubscriptionType};
use dyn_clone::DynClone;
use intercode_liquid::{tags::GraphQLExecutorBuilder, GraphQLExecutor};

use crate::{query_data::QueryData, schema_data::SchemaData};

pub trait RequestDataInjector: Send + Sync + DynClone {
  fn inject_data(&self, request: Request, query_data: &QueryData) -> Request;
}

pub struct EmbeddedGraphQLExecutor<Query, Mutation, Subscription> {
  schema: Schema<Query, Mutation, Subscription>,
  schema_data: SchemaData,
  query_data: QueryData,
  data_injector: Box<dyn RequestDataInjector>,
}

impl<Query, Mutation, Subscription> Debug
  for EmbeddedGraphQLExecutor<Query, Mutation, Subscription>
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("EmbeddedGraphQLExecutor")
      .field("schema_data", &self.schema_data)
      .field("query_data", &self.query_data)
      .finish_non_exhaustive()
  }
}

impl<Query, Mutation, Subscription> GraphQLExecutor
  for EmbeddedGraphQLExecutor<Query, Mutation, Subscription>
where
  Query: Send + Sync + ObjectType + 'static,
  Mutation: Send + Sync + ObjectType + 'static,
  Subscription: Send + Sync + SubscriptionType + 'static,
{
  fn execute(
    &self,
    request: async_graphql::Request,
  ) -> std::pin::Pin<Box<dyn Future<Output = async_graphql::Response> + Send + '_>> {
    let request = request.data(self.query_data.clone_ref());
    let request = self.data_injector.inject_data(request, &self.query_data);
    let response_future = async move { self.schema.execute(request).await };

    Box::pin(response_future)
  }
}

pub struct EmbeddedGraphQLExecutorBuilder<Query, Mutation, Subscription> {
  schema: Schema<Query, Mutation, Subscription>,
  query_data: QueryData,
  schema_data: SchemaData,
  data_injector: Box<dyn RequestDataInjector>,
}

impl<Query, Mutation, Subscription> Clone
  for EmbeddedGraphQLExecutorBuilder<Query, Mutation, Subscription>
{
  fn clone(&self) -> Self {
    Self {
      schema: self.schema.clone(),
      query_data: self.query_data.clone(),
      schema_data: self.schema_data.clone(),
      data_injector: dyn_clone::clone_box(self.data_injector.as_ref()),
    }
  }
}

impl<Query, Mutation, Subscription> EmbeddedGraphQLExecutorBuilder<Query, Mutation, Subscription> {
  pub fn new(
    schema: Schema<Query, Mutation, Subscription>,
    query_data: QueryData,
    schema_data: SchemaData,
    data_injector: Box<dyn RequestDataInjector>,
  ) -> Self {
    EmbeddedGraphQLExecutorBuilder {
      schema,
      query_data,
      schema_data,
      data_injector,
    }
  }
}

impl<Query, Mutation, Subscription> Debug
  for EmbeddedGraphQLExecutorBuilder<Query, Mutation, Subscription>
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("EmbeddedGraphQLExecutorBuilder")
      .field("query_data", &self.query_data)
      .field("schema_data", &self.schema_data)
      .finish_non_exhaustive()
  }
}

impl<Query, Mutation, Subscription> GraphQLExecutorBuilder
  for EmbeddedGraphQLExecutorBuilder<Query, Mutation, Subscription>
where
  Query: Send + Sync + ObjectType + 'static,
  Mutation: Send + Sync + ObjectType + 'static,
  Subscription: Send + Sync + SubscriptionType + 'static,
{
  fn build_executor(&self) -> Box<dyn GraphQLExecutor> {
    Box::new(EmbeddedGraphQLExecutor {
      schema: self.schema.clone(),
      query_data: self.query_data.clone_ref(),
      schema_data: self.schema_data.clone(),
      data_injector: dyn_clone::clone_box(self.data_injector.as_ref()),
    })
  }
}
