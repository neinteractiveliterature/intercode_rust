use async_graphql::Object;
use intercode_entities::PayWhatYouWantValue;

use super::MoneyType;

pub struct PayWhatYouWantValueType {
  value: PayWhatYouWantValue,
}

impl PayWhatYouWantValueType {
  pub fn new(value: PayWhatYouWantValue) -> Self {
    Self { value }
  }
}

#[Object(name = "PayWhatYouWantValue")]
impl PayWhatYouWantValueType {
  #[graphql(name = "minimum_amount")]
  pub async fn minimum_amount(&self) -> Option<MoneyType> {
    self.value.minimum_amount.clone().map(MoneyType::new)
  }

  #[graphql(name = "suggested_amount")]
  pub async fn suggested_amount(&self) -> Option<MoneyType> {
    self.value.suggested_amount.clone().map(MoneyType::new)
  }

  #[graphql(name = "maximum_amount")]
  pub async fn maximum_amount(&self) -> Option<MoneyType> {
    self.value.maximum_amount.clone().map(MoneyType::new)
  }
}
