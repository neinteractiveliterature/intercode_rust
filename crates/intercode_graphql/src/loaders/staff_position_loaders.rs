use intercode_entities::{
  links::StaffPositionToUserConProfiles, staff_positions, user_con_profiles,
};

impl_to_entity_id_loader!(staff_positions::Entity, staff_positions::PrimaryKey::Id);

impl_to_entity_link_loader!(
  staff_positions::Entity,
  StaffPositionToUserConProfiles,
  user_con_profiles::Entity,
  staff_positions::PrimaryKey::Id
);
