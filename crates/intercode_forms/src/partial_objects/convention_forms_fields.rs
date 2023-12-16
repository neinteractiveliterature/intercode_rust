use async_graphql::*;
use intercode_entities::{conventions, forms};
use intercode_graphql_core::{
  lax_id::LaxId, load_one_by_model_id, loader_result_to_required_single, model_backed_type,
  query_data::QueryData, ModelBackedType,
};
use sea_orm::{ColumnTrait, QueryFilter};

use super::FormFormsFields;

model_backed_type!(ConventionFormsFields, conventions::Model);

impl ConventionFormsFields {
  pub async fn form(&self, ctx: &Context<'_>, id: Option<ID>) -> Result<FormFormsFields> {
    let form = self
      .model
      .all_forms()
      .filter(forms::Column::Id.eq(LaxId::parse(id.clone().unwrap_or_default())?))
      .one(ctx.data::<QueryData>()?.db())
      .await?;
    form
      .ok_or_else(|| Error::new(format!("Form {:?} not found in convention", id)))
      .map(FormFormsFields::new)
  }

  pub async fn forms(&self, ctx: &Context<'_>) -> Result<Vec<FormFormsFields>> {
    let forms = self
      .model
      .all_forms()
      .all(ctx.data::<QueryData>()?.db())
      .await?;
    Ok(forms.into_iter().map(FormFormsFields::new).collect())
  }

  pub async fn user_con_profile_form(&self, ctx: &Context<'_>) -> Result<FormFormsFields> {
    let loader_result = load_one_by_model_id!(convention_user_con_profile_form, ctx, self)?;
    Ok(loader_result_to_required_single!(
      loader_result,
      FormFormsFields
    ))
  }
}
