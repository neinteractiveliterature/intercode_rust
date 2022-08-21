use intercode_entities::{links::ConventionToStaffPositions, *};

impl_to_entity_relation_loader!(
  conventions::Entity,
  event_categories::Entity,
  conventions::PrimaryKey::Id
);

impl_to_entity_link_loader!(
  conventions::Entity,
  ConventionToStaffPositions,
  staff_positions::Entity,
  conventions::PrimaryKey::Id
);

impl_to_entity_relation_loader!(
  conventions::Entity,
  ticket_types::Entity,
  conventions::PrimaryKey::Id
);

impl_to_entity_id_loader!(conventions::Entity, conventions::PrimaryKey::Id);
