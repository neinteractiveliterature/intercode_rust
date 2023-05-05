use async_graphql::Object;

pub struct MutationRoot;

#[Object(name = "Mutation")]
impl MutationRoot {
  async fn delete_me(&self) -> bool {
    true
  }
}
