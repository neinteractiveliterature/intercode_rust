use async_trait::async_trait;
use intercode_entities::model_ext::form_item_permissions::FormItemRole;
use sea_orm::ModelTrait;

use crate::Policy;

#[async_trait]
pub trait FormResponsePolicy<Principal: Send + Sync, Resource: ModelTrait + Sync>:
  Policy<Principal, Resource>
{
  async fn form_item_viewer_role(principal: &Principal, form_response: &Resource) -> FormItemRole;
  async fn form_item_writer_role(principal: &Principal, form_response: &Resource) -> FormItemRole;
}
