use intercode_entities::users;
use intercode_users::partial_objects::UserUsersFields;

use crate::merged_model_backed_type;

merged_model_backed_type!(UserType, users::Model, "User", UserUsersFields);
