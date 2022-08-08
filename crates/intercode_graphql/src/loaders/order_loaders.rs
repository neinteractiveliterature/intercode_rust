use intercode_entities::{order_entries, orders};

impl_to_entity_relation_loader!(
  orders::Entity,
  order_entries::Entity,
  orders::PrimaryKey::Id
);
