use quote::{quote, ToTokens};

use super::{implement_get_all_blocking::implement_get_all_blocking, LiquidDropImpl};

pub fn implement_object_view(liquid_drop_impl: &LiquidDropImpl) -> Box<dyn ToTokens> {
  let methods = liquid_drop_impl.methods.iter().collect::<Vec<_>>();
  let generics = &liquid_drop_impl.generics;
  let self_ty = &liquid_drop_impl.self_ty;
  let method_count = methods.len();
  let where_clause = &generics.where_clause;

  let getter_values = methods.iter().map(|method| {
    let ident = method.caching_getter_ident();
    quote!(#ident)
  });

  let method_name_strings: Vec<syn::LitStr> = methods
    .iter()
    .map(|getter_method| getter_method.name_str())
    .collect();

  let get_all_blocking = implement_get_all_blocking(methods.as_slice());

  let object_pairs = methods.iter().map(|method| {
    let ident = method.caching_getter_ident();
    let name_str = method.name_str();

    quote!(
      (#name_str, #ident)
    )
  });

  let object_getters = methods.iter().map(|method| {
    let ident = method.caching_getter_ident();
    let name_str = method.name_str();

    quote!(
      #name_str => Some(self.#ident().await.as_value())
    )
  });

  Box::new(quote!(
    impl #generics liquid::ObjectView for #self_ty #where_clause {
      fn as_value(&self) -> &dyn liquid::ValueView {
        self as &dyn liquid::ValueView
      }

      fn size(&self) -> i64 {
        usize::try_into(#method_count).unwrap()
      }

      fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = liquid::model::KStringCow<'k>> + 'k> {
        Box::new(
          vec![
            #(#method_name_strings),*
          ]
          .into_iter()
          .map(|s| s.into()),
        )
      }

      fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn liquid::ValueView> + 'k> {
        use ::seawater::LiquidDrop;
        #get_all_blocking
        let values: Vec<&dyn liquid::ValueView> = vec![
          #(#getter_values),*
        ];

        Box::new(values.into_iter().map(|drop_result| drop_result as &dyn ::liquid::ValueView))
      }

      fn iter<'k>(
        &'k self,
      ) -> Box<dyn Iterator<Item = (liquid::model::KStringCow<'k>, &'k dyn liquid::ValueView)> + 'k> {
        use ::seawater::LiquidDrop;
        #get_all_blocking
        let pairs: Vec<(&str, &dyn liquid::ValueView)> = vec![
          #(#object_pairs ,)*
        ];

        Box::new(
          pairs
            .into_iter()
            .map(|(key, value)| (key.into(), value as &dyn ::liquid::ValueView)),
        )
      }

      fn contains_key(&self, index: &str) -> bool {
        match index {
          #(#method_name_strings)|* => true,
          _ => false,
        }
      }

      fn get<'s>(&'s self, index: &str) -> Option<&'s dyn liquid::ValueView> {
        use ::seawater::LiquidDrop;
        tokio::task::block_in_place(move || {
          tokio::runtime::Handle::current().block_on(async move {
            match index {
              #(#object_getters ,)*
              _ => None,
            }
          })
        })
      }
    }
  ))
}
