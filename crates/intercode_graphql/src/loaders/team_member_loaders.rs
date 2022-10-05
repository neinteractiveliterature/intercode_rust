use intercode_entities::{events, team_members, user_con_profiles};

impl_to_entity_id_loader!(team_members::Entity, team_members::PrimaryKey::Id);

impl_to_entity_relation_loader!(
  team_members::Entity,
  events::Entity,
  team_members::PrimaryKey::Id
);

impl_to_entity_relation_loader!(
  team_members::Entity,
  user_con_profiles::Entity,
  team_members::PrimaryKey::Id
);
