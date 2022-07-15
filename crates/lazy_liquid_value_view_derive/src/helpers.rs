use quote::quote;
use syn::{
  parse::Parser, token::RArrow, Data, Error, Field, Path, PathArguments, ReturnType, Type,
  TypeGroup, TypeParamBound, TypePath,
};

pub fn extract_path_and_args(path: &Type) -> Option<(String, Option<PathArguments>)> {
  if let Type::Path(type_path) = path {
    Some((
      extract_type_path(type_path),
      extract_last_segment_arguments(type_path),
    ))
  } else if let Type::Reference(type_ref) = path {
    extract_path_and_args(type_ref.elem.as_ref())
  } else {
    None
  }
}

pub fn extract_return_type(output: &ReturnType) -> Option<(RArrow, String, Option<PathArguments>)> {
  let return_type = if let ReturnType::Type(rarrow, ref return_type) = output {
    Some((rarrow, return_type.as_ref().clone()))
  } else {
    None
  };

  if let Some((rarrow, the_type)) = return_type {
    extract_path_and_args(&the_type).map(|(path_only, last_segment_arguments)| {
      (rarrow.to_owned(), path_only, last_segment_arguments)
    })
  } else {
    None
  }
}

pub fn extract_type_path(type_path: &TypePath) -> String {
  type_path
    .path
    .segments
    .iter()
    .map(|segment| segment.ident.to_string())
    .collect::<Vec<_>>()
    .join("::")
}

pub fn extract_last_segment_arguments(type_path: &TypePath) -> Option<PathArguments> {
  type_path
    .path
    .segments
    .last()
    .map(|segment| segment.arguments.clone())
}

pub fn add_value_cell(data: &mut Data, value_type: &Path) {
  match data {
    Data::Struct(struct_data) => match &mut struct_data.fields {
      syn::Fields::Named(named_fields) => {
        named_fields.named.push(
          Field::parse_named
            .parse2(quote!(
              value_cell: tokio::sync::OnceCell<#value_type>
            ))
            .unwrap(),
        );
      }
      syn::Fields::Unnamed(_) => unimplemented!(),
      syn::Fields::Unit => unimplemented!(),
    },
    Data::Enum(_) | Data::Union(_) => unimplemented!(),
  }
}

// stolen from async_graphql_derive
pub fn get_type_path_and_name(ty: &Type) -> Result<(&Type, String), syn::Error> {
  match ty {
    Type::Path(path) => Ok((
      ty,
      path
        .path
        .segments
        .last()
        .map(|s| s.ident.to_string())
        .unwrap(),
    )),
    Type::Group(TypeGroup { elem, .. }) => get_type_path_and_name(elem),
    Type::TraitObject(trait_object) => Ok((
      ty,
      trait_object
        .bounds
        .iter()
        .find_map(|bound| match bound {
          TypeParamBound::Trait(t) => {
            Some(t.path.segments.last().map(|s| s.ident.to_string()).unwrap())
          }
          _ => None,
        })
        .unwrap(),
    )),
    _ => Err(Error::new_spanned(ty, "Invalid type")),
  }
}
