use std::collections::HashMap;

use syn::{AttrStyle, Attribute, Meta};

#[derive(Debug, Clone)]
pub enum DropMethodAttribute {
  Ignore,
  SerializeValue,
}

pub struct UnknownDropMethodAttribute;

impl TryFrom<&Attribute> for DropMethodAttribute {
  type Error = UnknownDropMethodAttribute;

  fn try_from(attr: &Attribute) -> Result<Self, Self::Error> {
    if attr.style == AttrStyle::Outer
      && attr
        .path
        .get_ident()
        .map(|ident| ident == "liquid_drop")
        .unwrap_or(false)
    {
      let meta_list = attr
        .parse_meta()
        .ok()
        .and_then(|parsed_meta| match parsed_meta {
          Meta::List(list) => Some(list),
          _ => None,
        });
      let nested_metas = meta_list.map(|list| {
        list
          .nested
          .into_iter()
          .flat_map(|item| match item {
            syn::NestedMeta::Meta(meta) => Some(meta),
            _ => None,
          })
          .collect::<Vec<_>>()
      });
      let name_values = nested_metas
        .iter()
        .flat_map(|metas| {
          metas
            .iter()
            .filter_map(|meta| match meta {
              Meta::NameValue(name_value) => Some(name_value),
              _ => None,
            })
            .filter_map(|name_value| {
              let path_ident = name_value.path.get_ident();
              path_ident.map(|ident| (ident.to_string(), name_value.lit.clone()))
            })
        })
        .collect::<HashMap<_, _>>();

      let ignore_flag = name_values
        .get("ignore")
        .and_then(|lit| match lit {
          syn::Lit::Bool(b) => Some(b.value),
          _ => None,
        })
        .unwrap_or(false);

      if ignore_flag {
        return Ok(DropMethodAttribute::Ignore);
      }

      let serialize_value_flag = name_values
        .get("serialize_value")
        .and_then(|lit| match lit {
          syn::Lit::Bool(b) => Some(b.value),
          _ => None,
        })
        .unwrap_or(false);

      if serialize_value_flag {
        return Ok(DropMethodAttribute::SerializeValue);
      }
    }

    Err(UnknownDropMethodAttribute)
  }
}
