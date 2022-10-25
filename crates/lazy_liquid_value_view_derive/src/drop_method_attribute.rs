use syn::{AttrStyle, Attribute, Meta};

#[derive(Debug, Clone)]
pub enum DropMethodAttribute {
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
      let serialize_value_flag = attr
        .parse_meta()
        .ok()
        .and_then(|parsed_meta| match parsed_meta {
          Meta::List(list) => Some(list),
          _ => None,
        })
        .map(|list| {
          list
            .nested
            .into_iter()
            .flat_map(|item| match item {
              syn::NestedMeta::Meta(meta) => Some(meta),
              _ => None,
            })
            .collect::<Vec<_>>()
        })
        .map(|metas| {
          metas.iter().any(|meta| match meta {
            Meta::NameValue(name_value) => name_value
              .path
              .get_ident()
              .map(|ident| ident == "serialize_value")
              .unwrap_or(false),
            _ => false,
          })
        })
        .unwrap_or(false);

      if serialize_value_flag {
        return Ok(DropMethodAttribute::SerializeValue);
      }
    }

    Err(UnknownDropMethodAttribute)
  }
}
