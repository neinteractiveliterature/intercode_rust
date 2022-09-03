use quote::{quote, ToTokens};

use crate::drop_getter_method::DropGetterMethod;

pub fn implement_get_all_blocking(methods: &[DropGetterMethod]) -> Box<dyn ToTokens> {
  let getter_invocations = methods.iter().map(|method| {
    let caching_getter_ident = method.caching_getter_ident();

    quote!(self.#caching_getter_ident())
  });

  let getter_idents = methods.iter().map(|method| method.ident());

  let destructure_var_names = getter_idents.clone();

  Box::new(quote!(
    let (#(#destructure_var_names ,)*) = tokio::task::block_in_place(move || {
      tokio::runtime::Handle::current().block_on(async move {
        futures::join!(
          #(#getter_invocations,)*
        )
      })
    });
  ))
}
