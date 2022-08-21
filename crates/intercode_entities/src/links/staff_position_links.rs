use crate::{staff_positions, staff_positions_user_con_profiles, user_con_profiles};
use sea_orm::{Linked, RelationDef, RelationTrait};

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
