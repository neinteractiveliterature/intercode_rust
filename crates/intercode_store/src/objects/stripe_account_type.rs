use async_graphql::*;

pub struct StripeAccountType {
  stripe_account: stripe::Account,
}

impl StripeAccountType {
  pub fn new(stripe_account: stripe::Account) -> Self {
    Self { stripe_account }
  }
}

#[Object(name = "StripeAccount")]
impl StripeAccountType {
  async fn id(&self) -> ID {
    self.stripe_account.id.clone().into()
  }

  #[graphql(name = "charges_enabled")]
  async fn charges_enabled(&self) -> bool {
    self.stripe_account.charges_enabled.unwrap_or(false)
  }

  #[graphql(name = "display_name")]
  async fn display_name(&self) -> Option<&str> {
    self
      .stripe_account
      .settings
      .as_ref()
      .and_then(|settings| settings.dashboard.display_name.as_deref())
  }

  async fn email(&self) -> Option<&str> {
    self.stripe_account.email.as_deref()
  }
}
