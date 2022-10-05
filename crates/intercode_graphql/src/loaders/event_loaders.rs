use intercode_entities::{event_categories, events, runs, team_members};

impl_to_entity_id_loader!(events::Entity, events::PrimaryKey::Id);

impl_to_entity_relation_loader!(
  events::Entity,
  event_categories::Entity,
  events::PrimaryKey::Id
);

impl_to_entity_relation_loader!(events::Entity, runs::Entity, events::PrimaryKey::Id);

impl_to_entity_relation_loader!(events::Entity, team_members::Entity, events::PrimaryKey::Id);
