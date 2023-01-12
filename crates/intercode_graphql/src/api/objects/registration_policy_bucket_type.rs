use async_graphql::*;
use intercode_entities::RegistrationPolicyBucket;

pub struct RegistrationPolicyBucketType(pub RegistrationPolicyBucket);

#[Object]
impl RegistrationPolicyBucketType {
  async fn key(&self) -> &str {
    &self.0.key
  }

  async fn name(&self) -> &str {
    &self.0.name
  }

  #[graphql(name = "not_counted")]
  async fn not_counted(&self) -> bool {
    self.0.is_not_counted()
  }

  #[graphql(name = "slots_limited")]
  async fn slots_limited(&self) -> bool {
    self.0.slots_limited()
  }

  #[graphql(name = "total_slots")]
  async fn total_slots(&self) -> Option<i32> {
    self.0.total_slots.clone().into()
  }
}
