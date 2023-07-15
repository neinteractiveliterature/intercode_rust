use once_cell::sync::Lazy;
use std::{env, sync::Arc};
use tracing::warn;

pub mod objects;
mod order_summary_presenter;
pub mod partial_objects;
pub mod policies;
pub mod query_builders;
pub mod unions;

static STRIPE_CLIENT: Lazy<Arc<stripe::Client>> = Lazy::new(|| {
  let secret_key = env::var("STRIPE_SECRET_KEY");
  let secret_key = match secret_key {
    Ok(value) => value,
    Err(err) => {
      warn!(
        "Could not get STRIPE_SECRET_KEY from environment ({}), Stripe API operations will fail",
        err
      );
      "".to_string()
    }
  };
  Arc::new(stripe::Client::new(secret_key))
});

pub fn inject_request_data(req: async_graphql::BatchRequest) -> async_graphql::BatchRequest {
  req.data::<Arc<stripe::Client>>(STRIPE_CLIENT.clone())
}
