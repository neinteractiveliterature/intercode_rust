use intercode_entities::{events, runs};

impl_to_entity_id_loader!(events::Entity, events::PrimaryKey::Id);

impl_to_entity_relation_loader!(events::Entity, runs::Entity, events::PrimaryKey::Id);
