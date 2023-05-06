use syn::{Attribute, LitBool};

#[derive(Debug, Clone)]
pub enum DropMethodAttribute {
  Ignore,
  SerializeValue,
}

pub struct UnknownDropMethodAttribute;

impl TryFrom<&Attribute> for DropMethodAttribute {
  type Error = UnknownDropMethodAttribute;

  fn try_from(attr: &Attribute) -> Result<Self, Self::Error> {
    let mut ignore_flag = false;
    let mut serialize_value_flag = false;

    if attr.path().is_ident("liquid_drop") {
      attr
        .parse_nested_meta(|meta| {
          if meta.path.is_ident("ignore") {
            let value = meta.value()?;
            let flag: LitBool = value.parse()?;
            ignore_flag = flag.value;
          } else if meta.path.is_ident("serialize_value") {
            let value = meta.value()?;
            let flag: LitBool = value.parse()?;
            serialize_value_flag = flag.value;
          }

          Ok(())
        })
        .map_err(|_err| UnknownDropMethodAttribute)?;
    }

    if ignore_flag {
      return Ok(Self::Ignore);
    }

    if serialize_value_flag {
      return Ok(Self::SerializeValue);
    }

    Err(UnknownDropMethodAttribute)
  }
}
