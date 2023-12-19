use async_graphql::*;
use async_trait::async_trait;
use intercode_entities::{conventions, forms};
use intercode_graphql_core::{
  lax_id::LaxId, load_one_by_model_id, loader_result_to_required_single, model_backed_type,
  query_data::QueryData, ModelBackedType,
};
use sea_orm::{ColumnTrait, QueryFilter};

model_backed_type!(ConventionFormsFields, conventions::Model);

#[async_trait]
pub trait ConventionFormsExtensions
where
  Self: ModelBackedType<Model = conventions::Model>,
{
  async fn form<T: ModelBackedType<Model = forms::Model>>(
    &self,
    ctx: &Context<'_>,
    id: Option<ID>,
  ) -> Result<T> {
    let form = self
      .get_model()
      .all_forms()
      .filter(forms::Column::Id.eq(LaxId::parse(id.clone().unwrap_or_default())?))
      .one(ctx.data::<QueryData>()?.db())
      .await?;
    form
      .ok_or_else(|| Error::new(format!("Form {:?} not found in convention", id)))
      .map(T::new)
  }

  async fn forms<T: ModelBackedType<Model = forms::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<T>> {
    let forms = self
      .get_model()
      .all_forms()
      .all(ctx.data::<QueryData>()?.db())
      .await?;
    Ok(forms.into_iter().map(T::new).collect())
  }

  async fn user_con_profile_form<T: ModelBackedType<Model = forms::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<T> {
    let loader_result = load_one_by_model_id!(convention_user_con_profile_form, ctx, self)?;
    Ok(loader_result_to_required_single!(loader_result, T))
  }
}
