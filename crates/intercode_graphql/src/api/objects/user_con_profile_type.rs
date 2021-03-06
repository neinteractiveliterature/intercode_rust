use crate::{
  loaders::{expect::ExpectModel, expect::ExpectModels},
  SchemaData,
};
use async_graphql::*;
use intercode_entities::{order_entries, orders, user_con_profiles, UserNames};
use pulldown_cmark::{html, Options, Parser};
use sea_orm::{sea_query::Expr, ColumnTrait, EntityTrait, QueryFilter};

use super::{
  ConventionType, ModelBackedType, OrderType, StaffPositionType, TeamMemberType, TicketType,
};
use crate::model_backed_type;
model_backed_type!(UserConProfileType, user_con_profiles::Model);

#[Object]
impl UserConProfileType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "accepted_clickwrap_agreement")]
  async fn accepted_clickwrap_agreement(&self) -> bool {
    self.model.accepted_clickwrap_agreement
  }

  #[graphql(name = "bio_html")]
  async fn bio_html(&self) -> Option<String> {
    if let Some(bio) = &self.model.bio {
      let mut options = Options::empty();
      options.insert(Options::ENABLE_STRIKETHROUGH);
      options.insert(Options::ENABLE_FOOTNOTES);
      options.insert(Options::ENABLE_SMART_PUNCTUATION);
      options.insert(Options::ENABLE_TABLES);
      let parser = Parser::new_ext(bio, options);

      let mut html_output = String::new();
      html::push_html(&mut html_output, parser);
      Some(html_output)
    } else {
      None
    }
  }

  #[graphql(name = "bio_name")]
  async fn bio_name(&self) -> String {
    self.model.bio_name()
  }

  async fn convention(&self, ctx: &Context<'_>) -> Result<ConventionType, Error> {
    let loader = &ctx.data::<SchemaData>()?.loaders.conventions_by_id;

    let model = loader
      .load_one(self.model.convention_id)
      .await?
      .expect_model()?;

    Ok(ConventionType::new(model))
  }

  #[graphql(name = "current_pending_order")]
  async fn current_pending_order(&self, ctx: &Context<'_>) -> Result<Option<OrderType>, Error> {
    // pending_orders = orders.pending.to_a
    let schema_data = ctx.data::<SchemaData>()?;
    let pending_orders = orders::Entity::find()
      .filter(
        orders::Column::UserConProfileId
          .eq(self.model.id)
          .and(orders::Column::Status.eq("pending")),
      )
      .all(schema_data.db.as_ref())
      .await?;

    if pending_orders.is_empty() {
      Ok(None)
    } else if pending_orders.len() > 1 {
      // combine orders into one cart
      let (first, rest) = pending_orders.split_at(1);
      order_entries::Entity::update_many()
        .col_expr(
          order_entries::Column::OrderId,
          Expr::value(sea_orm::Value::BigInt(Some(first[0].id))),
        )
        .filter(
          order_entries::Column::OrderId
            .is_in(rest.iter().map(|order| order.id).collect::<Vec<i64>>()),
        )
        .exec(schema_data.db.as_ref())
        .await?;

      Ok(Some(OrderType::new(first[0].to_owned())))
    } else {
      Ok(Some(OrderType::new(pending_orders[0].to_owned())))
    }
  }

  async fn email(&self, ctx: &Context<'_>) -> Result<String, Error> {
    let loader = &ctx.data::<SchemaData>()?.loaders.user_con_profile_user;

    Ok(
      loader
        .load_one(self.model.id)
        .await?
        .expect_one()?
        .email
        .to_owned(),
    )
  }

  #[graphql(name = "first_name")]
  async fn first_name(&self) -> &str {
    self.model.first_name.as_str()
  }

  #[graphql(name = "gravatar_enabled")]
  async fn gravatar_enabled(&self) -> bool {
    self.model.gravatar_enabled
  }

  #[graphql(name = "gravatar_url")]
  async fn gravatar_url(&self, ctx: &Context<'_>) -> Result<String, Error> {
    if self.model.gravatar_enabled {
      let loader = &ctx.data::<SchemaData>()?.loaders.users_by_id;

      let model = loader.load_one(self.model.user_id).await?.expect_model()?;
      Ok(format!(
        "https://gravatar.com/avatar/{:x}",
        md5::compute(model.email.trim().to_lowercase())
      ))
    } else {
      Ok(format!(
        "https://gravatar.com/avatar/{:x}",
        md5::compute("badrequest")
      ))
    }
  }

  #[graphql(name = "last_name")]
  async fn last_name(&self) -> &str {
    self.model.last_name.as_str()
  }

  #[graphql(name = "mobile_phone")]
  async fn mobile_phone(&self) -> Option<&str> {
    self.model.mobile_phone.as_deref()
  }

  async fn name(&self) -> String {
    self.model.name()
  }

  #[graphql(name = "name_inverted")]
  async fn name_inverted(&self) -> String {
    self.model.name_inverted()
  }

  #[graphql(name = "name_without_nickname")]
  async fn name_without_nickname(&self) -> String {
    self.model.name_without_nickname()
  }

  #[graphql(name = "staff_positions")]
  async fn staff_positions(&self, ctx: &Context<'_>) -> Result<Vec<StaffPositionType>, Error> {
    let loader = &ctx
      .data::<SchemaData>()?
      .loaders
      .user_con_profile_staff_positions;

    Ok(
      loader
        .load_one(self.model.id)
        .await?
        .expect_models()?
        .iter()
        .map(|staff_position| StaffPositionType::new(staff_position.to_owned()))
        .collect(),
    )
  }

  #[graphql(name = "team_members")]
  async fn team_members(&self, ctx: &Context<'_>) -> Result<Vec<TeamMemberType>, Error> {
    let loader = &ctx
      .data::<SchemaData>()?
      .loaders
      .user_con_profile_team_members;

    Ok(
      loader
        .load_one(self.model.id)
        .await?
        .expect_models()?
        .iter()
        .map(|team_member| TeamMemberType::new(team_member.to_owned()))
        .collect(),
    )
  }

  async fn ticket(&self, ctx: &Context<'_>) -> Result<Option<TicketType>, Error> {
    let loader = &ctx.data::<SchemaData>()?.loaders.user_con_profile_ticket;

    Ok(
      loader
        .load_one(self.model.id)
        .await?
        .try_one()
        .map(|ticket| TicketType::new(ticket.to_owned())),
    )
  }
}
