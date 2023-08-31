use async_graphql::*;
use intercode_entities::organizations;
use intercode_graphql_core::{query_data::QueryData, ModelBackedType};
use sea_orm::EntityTrait;

use super::{ConventionConventionsFields, OrganizationConventionsFields};

pub struct QueryRootConventionsFields;

impl QueryRootConventionsFields {
  pub async fn convention_by_request_host(
    ctx: &Context<'_>,
  ) -> Result<ConventionConventionsFields, Error> {
    let convention = Self::convention_by_request_host_if_present(ctx).await?;

    match convention {
      Some(convention) => Ok(convention),
      None => Err(Error::new("No convention found for this domain name")),
    }
  }

  pub async fn convention_by_request_host_if_present(
    ctx: &Context<'_>,
  ) -> Result<Option<ConventionConventionsFields>, Error> {
    let query_data = ctx.data::<QueryData>()?;

    match query_data.convention() {
      Some(convention) => Ok(Some(ConventionConventionsFields::new(
        convention.to_owned(),
      ))),
      None => Ok(None),
    }
  }

  pub async fn organizations(ctx: &Context<'_>) -> Result<Vec<OrganizationConventionsFields>> {
    let query_data = ctx.data::<QueryData>()?;

    Ok(
      organizations::Entity::find()
        .all(query_data.db())
        .await?
        .into_iter()
        .map(OrganizationConventionsFields::new)
        .collect(),
    )
  }
}
