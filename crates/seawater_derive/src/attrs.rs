use std::collections::HashMap;

use syn::{
  parse::{Parse, Parser},
  punctuated::Punctuated,
  spanned::Spanned,
  token::Comma,
  Error, Expr, Ident, Lit, Meta, MetaList, Path, Token,
};

pub struct RelatedAssociationMacroArgs {
  name: Ident,
  to: Path,
  inverse: Option<Ident>,
  serialize: bool,
  eager_load_associations: Vec<Ident>,
}

pub struct LinkedAssociationMacroArgs {
  name: Ident,
  to: Path,
  link: Path,
  inverse: Option<Ident>,
  serialize: bool,
  eager_load_associations: Vec<Ident>,
}

struct ArgsByType {
  path_args: Vec<Path>,
  list_args: HashMap<String, MetaList>,
  name_value_args: HashMap<String, Expr>,
}

pub trait AssociationMacroArgs {
  fn get_name(&self) -> &Ident;
  fn get_to(&self) -> &Path;
  fn get_inverse(&self) -> Option<&Ident>;
  fn get_link(&self) -> Option<&Path>;
  fn should_serialize(&self) -> bool;
  fn get_eager_load_associations(&self) -> &[Ident];
}

impl AssociationMacroArgs for RelatedAssociationMacroArgs {
  fn get_name(&self) -> &Ident {
    &self.name
  }

  fn get_to(&self) -> &Path {
    &self.to
  }

  fn get_inverse(&self) -> Option<&Ident> {
    self.inverse.as_ref()
  }

  fn get_link(&self) -> Option<&Path> {
    None
  }

  fn should_serialize(&self) -> bool {
    self.serialize
  }

  fn get_eager_load_associations(&self) -> &[Ident] {
    &self.eager_load_associations
  }
}

impl AssociationMacroArgs for LinkedAssociationMacroArgs {
  fn get_name(&self) -> &Ident {
    &self.name
  }

  fn get_to(&self) -> &Path {
    &self.to
  }

  fn get_inverse(&self) -> Option<&Ident> {
    self.inverse.as_ref()
  }

  fn get_link(&self) -> Option<&Path> {
    Some(&self.link)
  }

  fn should_serialize(&self) -> bool {
    self.serialize
  }

  fn get_eager_load_associations(&self) -> &[Ident] {
    &self.eager_load_associations
  }
}

fn start_parsing_args<'a>(
  vars_iter: &mut (dyn Iterator<Item = &'a Path> + 'a),
  input: &'a syn::parse::ParseBuffer,
) -> Result<(&'a Ident, &'a Path), Error> {
  let name = vars_iter
    .next()
    .ok_or_else(|| Error::new(input.span(), "Association name expected"))?
    .get_ident()
    .ok_or_else(|| Error::new(input.span(), "Not a valid identifier"))?;
  let to = vars_iter
    .next()
    .ok_or_else(|| Error::new(input.span(), "Target drop expected"))?;
  Ok((name, to))
}

fn split_attribute_args(args: Punctuated<Meta, Token![,]>) -> ArgsByType {
  let path_args = args
    .iter()
    .filter_map(|var| {
      if let Meta::Path(path) = var {
        Some(path)
      } else {
        None
      }
    })
    .cloned()
    .collect::<Vec<_>>();

  let list_args = args
    .iter()
    .filter_map(|var| {
      if let Meta::List(list) = var {
        Some(list)
      } else {
        None
      }
    })
    .filter_map(|list| {
      list
        .path
        .get_ident()
        .map(|ident| (ident.to_string(), list.clone()))
    })
    .collect::<HashMap<_, _>>();

  let name_value_args = args
    .iter()
    .filter_map(|arg| {
      if let Meta::NameValue(pair) = arg {
        Some(pair)
      } else {
        None
      }
    })
    .filter_map(|pair| {
      pair
        .path
        .get_ident()
        .map(|ident| (ident.to_string(), pair.value.clone()))
    })
    .collect::<HashMap<_, _>>();

  ArgsByType {
    path_args,
    list_args,
    name_value_args,
  }
}

impl Parse for RelatedAssociationMacroArgs {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let vars = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;
    let args_by_type = split_attribute_args(vars);

    let mut path_args_iter = args_by_type.path_args.iter();

    let (name, to) = start_parsing_args(&mut path_args_iter, input)?;
    if path_args_iter.next().is_some() {
      return Err(Error::new(
        input.span(),
        "Unexpected parameter for association macro",
      ));
    }

    let (inverse, serialize, eager_load_associations) = parse_optional_args(&args_by_type)?;

    Ok(RelatedAssociationMacroArgs {
      name: name.to_owned(),
      to: to.to_owned(),
      inverse,
      serialize,
      eager_load_associations,
    })
  }
}

impl Parse for LinkedAssociationMacroArgs {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let vars = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;
    let args_by_type = split_attribute_args(vars);

    let mut path_args_iter = args_by_type.path_args.iter();

    let (name, to) = start_parsing_args(&mut path_args_iter, input)?;
    let link = path_args_iter
      .next()
      .ok_or_else(|| Error::new(input.span(), "Link expected"))?;
    if path_args_iter.next().is_some() {
      return Err(Error::new(
        input.span(),
        "Unexpected parameter for association macro",
      ));
    }

    let (inverse, serialize, eager_load_associations) = parse_optional_args(&args_by_type)?;

    Ok(LinkedAssociationMacroArgs {
      name: name.to_owned(),
      to: to.to_owned(),
      link: link.to_owned(),
      inverse,
      serialize,
      eager_load_associations,
    })
  }
}

fn parse_optional_args(
  args_by_type: &ArgsByType,
) -> Result<(Option<Ident>, bool, Vec<Ident>), Error> {
  let inverse = args_by_type
    .list_args
    .get("inverse")
    .map(|list| Ident::parse.parse2(list.tokens.clone()))
    .transpose()?;

  let eager_load_associations = args_by_type
    .list_args
    .get("eager_load")
    .map(|list| Punctuated::<Ident, Comma>::parse_terminated.parse2(list.tokens.clone()))
    .transpose()?
    .unwrap_or_default()
    .into_iter()
    .collect::<Vec<_>>();

  let serialize = args_by_type
    .name_value_args
    .get("serialize")
    .map(|expr| {
      if let Expr::Lit(lit) = expr {
        if let Lit::Bool(bool_value) = &lit.lit {
          return Ok(bool_value.value);
        }
      }
      Err(Error::new(expr.span(), "Boolean value expected"))
    })
    .transpose()?
    .unwrap_or(false);

  Ok((inverse, serialize, eager_load_associations))
}
