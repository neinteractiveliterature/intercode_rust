use async_graphql::*;
use intercode_entities::{conventions, organizations};
use intercode_graphql_core::{
  lax_id::LaxId, query_data::QueryData, ModelBackedType, ModelPaginator,
};
use intercode_query_builders::{sort_input::SortInput, PaginationFromQueryBuilder};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::query_builders::{ConventionFiltersInput, ConventionsQueryBuilder};

pub struct QueryRootConventionsFields;

impl QueryRootConventionsFields {
  pub async fn convention_by_domain<T: ModelBackedType<Model = conventions::Model>>(
    ctx: &Context<'_>,
    domain: String,
  ) -> Result<T, Error> {
    let query_data = ctx.data::<QueryData>()?;
    let convention = conventions::Entity::find()
      .filter(conventions::Column::Domain.eq(domain))
      .one(query_data.db())
      .await?;

    match convention {
      Some(convention) => Ok(T::new(convention)),
      None => Err(Error::new("No convention found for this domain name")),
    }
  }

  pub async fn convention_by_id<T: ModelBackedType<Model = conventions::Model>>(
    ctx: &Context<'_>,
    id: ID,
  ) -> Result<T, Error> {
    let query_data = ctx.data::<QueryData>()?;
    let convention = conventions::Entity::find()
      .filter(conventions::Column::Id.eq(LaxId::parse(id)?))
      .one(query_data.db())
      .await?;

    match convention {
      Some(convention) => Ok(T::new(convention)),
      None => Err(Error::new("No convention found for this ID")),
    }
  }

  pub async fn convention_by_request_host<T: ModelBackedType<Model = conventions::Model>>(
    ctx: &Context<'_>,
  ) -> Result<T, Error> {
    let convention = Self::convention_by_request_host_if_present(ctx).await?;

    match convention {
      Some(convention) => Ok(convention),
      None => Err(Error::new("No convention found for this domain name")),
    }
  }

  pub async fn convention_by_request_host_if_present<
    T: ModelBackedType<Model = conventions::Model>,
  >(
    ctx: &Context<'_>,
  ) -> Result<Option<T>, Error> {
    let query_data = ctx.data::<QueryData>()?;

    match query_data.convention() {
      Some(convention) => Ok(Some(T::new(convention.to_owned()))),
      None => Ok(None),
    }
  }

  pub async fn conventions_paginated<T: ModelBackedType<Model = conventions::Model>>(
    filters: Option<ConventionFiltersInput>,
    sorts: Option<Vec<SortInput>>,
    page: Option<u64>,
    per_page: Option<u64>,
  ) -> ModelPaginator<T> {
    ModelPaginator::from_query_builder(
      &ConventionsQueryBuilder::new(filters, sorts),
      conventions::Entity::find(),
      page,
      per_page,
    )
  }

  pub async fn organizations<T: ModelBackedType<Model = organizations::Model>>(
    ctx: &Context<'_>,
  ) -> Result<Vec<T>> {
    let query_data = ctx.data::<QueryData>()?;

    Ok(
      organizations::Entity::find()
        .all(query_data.db())
        .await?
        .into_iter()
        .map(T::new)
        .collect(),
    )
  }
}
