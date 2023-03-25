use async_graphql::indexmap::IndexMap;
use futures::try_join;
use intercode_entities::{conventions, events};
use intercode_graphql::QueryData;
use liquid::ValueView;
use once_cell::race::OnceBox;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use seawater::{liquid_drop_impl, Context, DropError, ModelBackedDrop};

use super::{ConventionDrop, DropContext, EventDrop, UserConProfileDrop};

#[derive(Debug)]
pub struct IntercodeGlobals {
  query_data: QueryData,
  context: DropContext,
  _liquid_object_view_pairs: OnceBox<IndexMap<String, Box<dyn ValueView + Send + Sync>>>,
}

impl Clone for IntercodeGlobals {
  fn clone(&self) -> Self {
    Self {
      query_data: self.query_data.clone(),
      context: self.context.clone(),
      _liquid_object_view_pairs: OnceBox::new(),
    }
  }
}

#[liquid_drop_impl(i64, DropContext)]
impl IntercodeGlobals {
  pub fn new(context: DropContext) -> Self {
    IntercodeGlobals {
      query_data: context.query_data().clone(),
      context,
      _liquid_object_view_pairs: OnceBox::new(),
    }
  }

  fn id(&self) -> i64 {
    0
  }

  fn convention(&self) -> Option<ConventionDrop> {
    self
      .query_data
      .convention()
      .map(|convention| ConventionDrop::new(convention.clone(), self.context.clone()))
  }

  async fn conventions(&self) -> Result<Vec<ConventionDrop>, DropError> {
    Ok(
      conventions::Entity::find()
        .filter(conventions::Column::Hidden.eq(false))
        .all(self.context.db())
        .await?
        .iter()
        .map(|convention| ConventionDrop::new(convention.clone(), self.context.clone()))
        .collect(),
    )
  }

  async fn event(&self) -> Option<EventDrop> {
    if let Some(convention) = self.query_data.convention() {
      if convention.site_mode == "single_event" {
        return events::Entity::find()
          .filter(events::Column::ConventionId.eq(convention.id))
          .one(self.context.db())
          .await
          .ok()
          .flatten()
          .map(|event| EventDrop::new(event, self.context.clone()));
      }
    }

    None
  }

  async fn user_con_profile(&self) -> Option<UserConProfileDrop> {
    let ucp = self.query_data.user_con_profile().map(|user_con_profile| {
      UserConProfileDrop::new(user_con_profile.clone(), self.context.clone())
    });

    if let Some(ucp) = ucp {
      let ucp_ref = self
        .context
        .with_drop_store(|store| store.store(ucp.clone()));
      let drops = vec![ucp_ref];
      try_join!(
        UserConProfileDrop::preload_signups(self.context.clone(), &drops),
        UserConProfileDrop::preload_staff_positions(self.context.clone(), &drops),
        UserConProfileDrop::preload_ticket(self.context.clone(), &drops),
        UserConProfileDrop::preload_user(self.context.clone(), &drops),
      );
      Some(ucp)
    } else {
      None
    }
  }
}
