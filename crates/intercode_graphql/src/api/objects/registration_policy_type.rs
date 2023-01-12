use super::RegistrationPolicyBucketType;
use async_graphql::*;
use intercode_entities::RegistrationPolicy;

pub struct RegistrationPolicyType(pub RegistrationPolicy);

#[Object]
impl RegistrationPolicyType {
  async fn buckets(&self) -> Vec<RegistrationPolicyBucketType> {
    self
      .0
      .all_buckets()
      .cloned()
      .map(RegistrationPolicyBucketType)
      .collect()
  }

  #[graphql(name = "minimum_slots")]
  async fn minimum_slots(&self) -> Option<i32> {
    self.0.minimum_slots().into()
  }

  #[graphql(name = "minimum_slots_including_not_counted")]
  async fn minimum_slots_including_not_counted(&self) -> Option<i32> {
    self.0.minimum_slots_including_not_counted().into()
  }

  #[graphql(name = "only_uncounted")]
  async fn only_uncounted(&self) -> bool {
    self.0.only_uncounted()
  }

  #[graphql(name = "preferred_slots")]
  async fn preferred_slots(&self) -> Option<i32> {
    self.0.preferred_slots().into()
  }

  #[graphql(name = "preferred_slots_including_not_counted")]
  async fn preferred_slots_including_not_counted(&self) -> Option<i32> {
    self.0.preferred_slots_including_not_counted().into()
  }

  #[graphql(name = "slots_limited")]
  async fn slots_limited(&self) -> bool {
    self.0.slots_limited()
  }

  #[graphql(name = "total_slots")]
  async fn total_slots(&self) -> Option<i32> {
    self.0.total_slots().into()
  }

  #[graphql(name = "total_slots_including_not_counted")]
  async fn total_slots_including_not_counted(&self) -> Option<i32> {
    self.0.total_slots_including_not_counted().into()
  }
}
