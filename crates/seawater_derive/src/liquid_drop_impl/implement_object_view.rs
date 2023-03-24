use quote::{quote, ToTokens};

use super::LiquidDropImpl;

pub fn implement_object_view(liquid_drop_impl: &LiquidDropImpl) -> Box<dyn ToTokens> {
  let methods = liquid_drop_impl.methods.iter().collect::<Vec<_>>();
  let generics = &liquid_drop_impl.generics;
  let self_ty = &liquid_drop_impl.self_ty;
  let method_count = methods.len();
  let where_clause = &generics.where_clause;

  let method_name_strings: Vec<syn::LitStr> = methods
    .iter()
    .map(|getter_method| getter_method.name_str())
    .collect();

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

      fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn ::liquid::ValueView> + 'k> {
        Box::new(
          self
            .get_all_blocking()
            .iter()
            .map(|(_key, value)| value.as_ref() as &dyn liquid::ValueView),
        )
      }

      fn iter<'k>(
        &'k self,
      ) -> Box<dyn Iterator<Item = (::liquid::model::KStringCow<'k>, &'k dyn ::liquid::ValueView)> + 'k> {
        Box::new(self.get_all_blocking().iter().map(|(key, value)| {
          (
            ::liquid::model::KStringCow::from_ref(key),
            value.as_ref() as &dyn ::liquid::ValueView,
          )
        }))
      }

      fn contains_key(&self, index: &str) -> bool {
        match index {
          #(#method_name_strings)|* => true,
          _ => false,
        }
      }

      fn get<'s>(&'s self, index: &str) -> Option<&'s dyn liquid::ValueView> {
        let index_map = self.get_all_blocking();
        index_map.get(index).map(|boxed| boxed.as_ref() as &dyn ::liquid::ValueView)
      }
    }
  ))
}
