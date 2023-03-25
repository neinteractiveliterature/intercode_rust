use quote::{quote, ToTokens};

use crate::drop_getter_method::DropGetterMethod;

pub fn implement_get_all_blocking(methods: &[&DropGetterMethod]) -> Box<dyn ToTokens> {
  let getter_invocations = methods.iter().map(|method| {
    let caching_getter_ident = method.caching_getter_ident();

    quote!(self.#caching_getter_ident())
  });

  let getter_idents = methods.iter().map(|method| method.caching_getter_ident());

  let map_pairs = methods
    .iter()
    .map(|method| (method.name_str(), method.caching_getter_ident()))
    .map(|(key, value)| {
      quote!(
        (
          #key.to_string(),
          Box::new(#value) as Box<dyn liquid::ValueView + Send + Sync>,
        )
      )
    });

  Box::new(quote!(
    fn get_all_blocking(&self) -> &::async_graphql::indexmap::IndexMap<String, Box<dyn liquid::ValueView + Send + Sync>> {
      self._liquid_object_view_pairs.get_or_init(|| {
        let (#(#getter_idents ,)*) = tokio::task::block_in_place(move || {
          tokio::runtime::Handle::current()
            .block_on(async move { ::futures::join![#(#getter_invocations ,)*] })
        });

        Box::new(
          vec![#(#map_pairs ,)*].into_iter().collect(),
        )
      })
    }
  ))
}
