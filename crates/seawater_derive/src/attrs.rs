use std::collections::HashMap;

use syn::{
  parse::Parse, punctuated::Punctuated, spanned::Spanned, token::Comma, Error, Ident, Lit, Meta,
  NestedMeta, Path, Token,
};

use crate::associations::EagerLoadAssociation;

pub struct RelatedAssociationMacroArgs {
  name: Ident,
  to: Path,
  inverse: Option<Ident>,
  serialize: bool,
  eager_load_associations: Vec<EagerLoadAssociation>,
}

pub struct LinkedAssociationMacroArgs {
  name: Ident,
  to: Path,
  link: Path,
  inverse: Option<Ident>,
  serialize: bool,
  eager_load_associations: Vec<EagerLoadAssociation>,
}

struct ArgsByType {
  path_args: Vec<Path>,
  list_args: HashMap<String, Punctuated<NestedMeta, Comma>>,
  name_value_args: HashMap<String, Lit>,
}

pub trait AssociationMacroArgs {
  fn get_name(&self) -> &Ident;
  fn get_to(&self) -> &Path;
  fn get_inverse(&self) -> Option<&Ident>;
  fn get_link(&self) -> Option<&Path>;
  fn should_serialize(&self) -> bool;
  fn get_eager_load_associations(&self) -> &[EagerLoadAssociation];
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

  fn get_eager_load_associations(&self) -> &[EagerLoadAssociation] {
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

  fn get_eager_load_associations(&self) -> &[EagerLoadAssociation] {
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
        .map(|ident| (ident.to_string(), list.nested.clone()))
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
        .map(|ident| (ident.to_string(), pair.lit.clone()))
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

fn parse_eager_load_association(meta: &NestedMeta) -> Result<EagerLoadAssociation, Error> {
  match meta {
    NestedMeta::Meta(Meta::Path(path)) => path
      .get_ident()
      .cloned()
      .ok_or_else(|| Error::new(path.span(), "Not a valid identifier"))
      .map(|ident| EagerLoadAssociation {
        ident,
        children: vec![],
      }),
    NestedMeta::Meta(Meta::List(list)) => {
      let ident = list
        .path
        .get_ident()
        .cloned()
        .ok_or_else(|| Error::new(list.path.span(), "Not a valid identifier"));

      ident.and_then(|ident| {
        let mut children: Vec<EagerLoadAssociation> = Vec::with_capacity(list.nested.len());
        for nested in list.nested.iter() {
          let parse_result = parse_eager_load_association(nested);
          match parse_result {
            Ok(child) => children.push(child),
            Err(e) => return Err(e),
          }
        }
        Ok(EagerLoadAssociation { ident, children })
      })
    }
    _ => Err(Error::new(meta.span(), "Identifier or list expected")),
  }
}

fn parse_optional_args(
  args_by_type: &ArgsByType,
) -> Result<(Option<Ident>, bool, Vec<EagerLoadAssociation>), Error> {
  let inverse = args_by_type
    .list_args
    .get("inverse")
    .map(|nested| {
      let mut nested_iter = nested.iter();
      let path = nested_iter
        .next()
        .map(|meta| match meta {
          NestedMeta::Meta(Meta::Path(path)) => path
            .get_ident()
            .cloned()
            .ok_or_else(|| Error::new(path.span(), "Not a valid identifier")),
          _ => Err(Error::new(meta.span(), "Identifier expected")),
        })
        .transpose();

      if nested_iter.next().is_some() {
        Err(Error::new(
          nested.span(),
          "Unexpected parameter for inverse",
        ))
      } else {
        path
      }
    })
    .transpose()?
    .flatten();

  let eager_load_associations = args_by_type
    .list_args
    .get("eager_load")
    .map(|nested| {
      let nested_iter = nested.iter();
      nested_iter
        .map(parse_eager_load_association)
        .collect::<Result<Vec<_>, _>>()
    })
    .transpose()?
    .unwrap_or_default();

  let serialize = args_by_type
    .name_value_args
    .get("serialize")
    .map(|lit| {
      if let Lit::Bool(bool_value) = lit {
        Ok(bool_value.value)
      } else {
        Err(Error::new(lit.span(), "Boolean value expected"))
      }
    })
    .transpose()?
    .unwrap_or(false);

  Ok((inverse, serialize, eager_load_associations))
}
