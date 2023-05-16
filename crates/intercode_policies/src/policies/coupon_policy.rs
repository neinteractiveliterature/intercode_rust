use axum::async_trait;
use intercode_entities::{conventions, coupon_applications, coupons, orders};
use sea_orm::{ColumnTrait, DbErr, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect};

use crate::{
  authorization_info::AuthorizationInfo,
  policy::{EntityPolicy, Policy, ReadManageAction},
};

pub struct CouponPolicy;

#[async_trait]
impl Policy<AuthorizationInfo, coupons::Model> for CouponPolicy {
  type Action = ReadManageAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &ReadManageAction,
    resource: &coupons::Model,
  ) -> Result<bool, Self::Error> {
    if !principal.can_act_in_convention(resource.convention_id) {
      return Ok(false);
    }

    match action {
      ReadManageAction::Read => {
        // Coupons are special; they are by definition semi-secret.  So we use the "update_products"
        // permission as a proxy for "can see the existence of coupons".
        if principal.has_scope("read_conventions") {
          let convention_id = resource.convention_id;
          let has_permission = principal
            .has_convention_permission("update_coupons", convention_id)
            .await?;
          let has_applied_coupon = coupon_applications::Entity::find()
            .filter(coupon_applications::Column::CouponId.eq(resource.id))
            .inner_join(orders::Entity)
            .filter(
              orders::Column::UserConProfileId
                .is_in(principal.user_con_profile_ids().await?.clone()),
            )
            .count(&principal.db)
            .await?
            > 0;
          Ok(has_permission || has_applied_coupon || principal.site_admin_manage())
        } else {
          Ok(false)
        }
      }
      ReadManageAction::Manage => {
        if principal.has_scope("manage_conventions") {
          let convention_id = resource.convention_id;
          let has_permission = principal
            .has_convention_permission("update_coupons", convention_id)
            .await?;
          Ok(has_permission || principal.site_admin_manage())
        } else {
          Ok(false)
        }
      }
    }
  }
}

impl EntityPolicy<AuthorizationInfo, coupons::Model> for CouponPolicy {
  type Action = ReadManageAction;
  fn accessible_to(
    principal: &AuthorizationInfo,
    _action: &Self::Action,
  ) -> sea_orm::Select<<coupons::Model as sea_orm::ModelTrait>::Entity> {
    coupons::Entity::find().filter(
      coupons::Column::ConventionId.in_subquery(
        QuerySelect::query(
          &mut principal
            .conventions_with_permission("update_products")
            .select_only()
            .column(conventions::Column::Id),
        )
        .take(),
      ),
    )
  }
}
