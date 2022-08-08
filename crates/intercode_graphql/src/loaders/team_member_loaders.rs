use intercode_entities::{events, team_members};

impl_to_entity_id_loader!(team_members::Entity, team_members::PrimaryKey::Id);

impl_to_entity_relation_loader!(
  team_members::Entity,
  events::Entity,
  team_members::PrimaryKey::Id
);
