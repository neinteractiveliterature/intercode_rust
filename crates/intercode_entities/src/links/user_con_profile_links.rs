use sea_orm::{Linked, RelationDef, RelationTrait};

use crate::{staff_positions, staff_positions_user_con_profiles, user_con_profiles};

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
