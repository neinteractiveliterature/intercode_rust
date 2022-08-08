use intercode_entities::{products, ticket_types};

impl_to_entity_relation_loader!(
  ticket_types::Entity,
  products::Entity,
  ticket_types::PrimaryKey::Id
);
