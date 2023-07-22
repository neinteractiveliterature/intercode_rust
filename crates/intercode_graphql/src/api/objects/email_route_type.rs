use async_graphql::*;
use intercode_entities::email_routes;
use intercode_graphql_core::model_backed_type;
use intercode_policies::{
  policies::EmailRoutePolicy, ModelBackedTypeGuardablePolicy, ReadManageAction,
};

model_backed_type!(EmailRouteType, email_routes::Model);

#[Object(
  name = "EmailRoute",
  guard = "EmailRoutePolicy::model_guard(ReadManageAction::Read, self)"
)]
impl EmailRouteType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "forward_addresses")]
  async fn forward_addresses(&self) -> &Vec<String> {
    &self.model.forward_addresses
  }

  #[graphql(name = "receiver_address")]
  async fn receive_address(&self) -> &str {
    &self.model.receiver_address
  }
}
