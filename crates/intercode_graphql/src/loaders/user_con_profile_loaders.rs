use intercode_entities::{
  signups, staff_positions, staff_positions_user_con_profiles, team_members, tickets,
  user_con_profiles, users,
};
use sea_orm::{Linked, RelationDef, RelationTrait};

#[derive(Debug, Clone)]
pub struct UserConProfileToStaffPositions;

impl Linked for UserConProfileToStaffPositions {
  type FromEntity = user_con_profiles::Entity;
  type ToEntity = staff_positions::Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![
      staff_positions_user_con_profiles::Relation::UserConProfiles
        .def()
        .rev(),
      staff_positions_user_con_profiles::Relation::StaffPositions.def(),
    ]
  }
}

impl_to_entity_relation_loader!(
  user_con_profiles::Entity,
  team_members::Entity,
  user_con_profiles::PrimaryKey::Id
);

impl_to_entity_relation_loader!(
  user_con_profiles::Entity,
  signups::Entity,
  user_con_profiles::PrimaryKey::Id
);

impl_to_entity_link_loader!(
  user_con_profiles::Entity,
  UserConProfileToStaffPositions,
  staff_positions::Entity,
  user_con_profiles::PrimaryKey::Id
);

impl_to_entity_relation_loader!(
  user_con_profiles::Entity,
  tickets::Entity,
  user_con_profiles::PrimaryKey::Id
);

impl_to_entity_relation_loader!(
  user_con_profiles::Entity,
  users::Entity,
  user_con_profiles::PrimaryKey::Id
);
