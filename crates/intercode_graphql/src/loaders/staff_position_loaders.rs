use intercode_entities::{staff_positions, staff_positions_user_con_profiles, user_con_profiles};
use sea_orm::{Linked, RelationDef, RelationTrait};

impl_to_entity_id_loader!(staff_positions::Entity, staff_positions::PrimaryKey::Id);

#[derive(Debug, Clone)]
pub struct StaffPositionToUserConProfiles;

impl Linked for StaffPositionToUserConProfiles {
  type FromEntity = staff_positions::Entity;
  type ToEntity = user_con_profiles::Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![
      staff_positions_user_con_profiles::Relation::StaffPositions
        .def()
        .rev(),
      staff_positions_user_con_profiles::Relation::UserConProfiles.def(),
    ]
  }
}

impl_to_entity_link_loader!(
  staff_positions::Entity,
  StaffPositionToUserConProfiles,
  user_con_profiles::Entity,
  staff_positions::PrimaryKey::Id
);
