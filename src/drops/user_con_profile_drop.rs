use futures::try_join;
use intercode_entities::{links::UserConProfileToStaffPositions, user_con_profiles, UserNames};
use intercode_graphql::SchemaData;
use intercode_inflector::IntercodeInflector;
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
use seawater::{
  belongs_to_related, has_many_linked, has_many_related, has_one_related, model_backed_drop,
  preloaders::Preloader, DropError,
};

use super::{SignupDrop, StaffPositionDrop, TicketDrop, UserDrop};

model_backed_drop!(UserConProfileDrop, user_con_profiles::Model);

#[has_many_related(signups, SignupDrop)]
#[has_many_linked(staff_positions, StaffPositionDrop, UserConProfileToStaffPositions)]
#[has_one_related(ticket, TicketDrop)]
#[belongs_to_related(user, UserDrop)]
#[liquid_drop_impl]
impl UserConProfileDrop {
  pub fn id(&self) -> i64 {
    self.model.id
  }

  fn first_name(&self) -> &str {
    self.model.first_name.as_str()
  }

  fn ical_secret(&self) -> &str {
    self.model.ical_secret.as_str()
  }

  fn last_name(&self) -> &str {
    self.model.last_name.as_str()
  }

  fn name_without_nickname(&self) -> String {
    self.model.name_without_nickname()
  }

  async fn privileges(&self) -> Result<Vec<String>, DropError> {
    let inflector = IntercodeInflector::new();

    Ok(
      self
        .caching_user()
        .await
        .get_inner()
        .unwrap()
        .privileges()
        .iter()
        .map(|priv_name| inflector.humanize(priv_name))
        .collect::<Vec<_>>(),
    )
  }

  pub async fn preload_users_and_signups(
    schema_data: SchemaData,
    drops: &[&UserConProfileDrop],
  ) -> Result<(), DropError> {
    try_join!(
      async {
        Self::user_preloader()
          .preload(schema_data.db.as_ref(), drops)
          .await?;
        Ok::<(), DropError>(())
      },
      async {
        Self::signups_preloader()
          .preload(schema_data.db.as_ref(), drops)
          .await?;
        Ok::<(), DropError>(())
      }
    )?;

    Ok(())
  }
}
