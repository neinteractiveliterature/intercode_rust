use intercode_entities::staff_positions;
use intercode_graphql::{loaders::expect::ExpectModels, SchemaData};
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};

use super::{DropError, UserConProfileDrop};

#[liquid_drop_struct]
pub struct StaffPositionDrop {
  schema_data: SchemaData,
  staff_position: staff_positions::Model,
}

#[liquid_drop_impl]
impl StaffPositionDrop {
  pub fn new(staff_position: staff_positions::Model, schema_data: SchemaData) -> Self {
    StaffPositionDrop {
      schema_data,
      staff_position,
    }
  }

  fn id(&self) -> i64 {
    self.staff_position.id
  }

  fn email(&self) -> Option<&str> {
    self.staff_position.email.as_deref()
  }

  fn email_link(&self) -> Option<String> {
    self
      .email()
      .map(|email| format!("<a href=\"mailto:{}\">{}</a>", email, email))
  }

  fn name(&self) -> Option<&str> {
    self.staff_position.name.as_deref()
  }

  async fn user_con_profiles(&self) -> Result<Vec<UserConProfileDrop<'cache>>, DropError> {
    Ok(
      self
        .schema_data
        .loaders
        .staff_position_user_con_profiles
        .load_one(self.staff_position.id)
        .await?
        .expect_models()?
        .iter()
        .map(|user_con_profile| {
          UserConProfileDrop::new(user_con_profile.clone(), self.schema_data.clone())
        })
        .collect::<Vec<_>>(),
    )
  }
}
