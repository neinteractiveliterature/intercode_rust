use intercode_entities::users;

impl_to_entity_id_loader!(users::Entity, users::PrimaryKey::Id);
