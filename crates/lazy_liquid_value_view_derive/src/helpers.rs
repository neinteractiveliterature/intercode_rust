use quote::quote;
use syn::{
  parse::Parser, parse_quote, punctuated::Punctuated, token::Comma, Data, Error, Field,
  GenericArgument, GenericParam, Path, PathArguments, PathSegment, Type, TypeGroup, TypeParamBound,
  TypePath,
};

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
pub fn get_type_path_and_name_and_arguments(
  ty: &Type,
) -> Result<(Type, String, Option<PathArguments>), syn::Error> {
  match ty {
    Type::Path(path) => Ok((
      ty.clone(),
      path
        .path
        .segments
        .last()
        .map(|s| s.ident.to_string())
        .unwrap(),
      path.path.segments.last().map(|s| s.arguments.clone()),
    )),
    Type::Group(TypeGroup { elem, .. }) => get_type_path_and_name_and_arguments(elem),
    Type::TraitObject(trait_object) => Ok((
      ty.clone(),
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
      None,
    )),
    _ => Err(Error::new_spanned(ty, "Invalid type")),
  }
}

pub fn eliminate_references(ty: Box<Type>) -> Box<Type> {
  if let Type::Reference(ref_type) = *ty {
    if let Type::Path(elem_path) = *ref_type.elem.clone() {
      if elem_path.path.is_ident("str") {
        let new_type: Type = parse_quote!(String);
        return Box::new(new_type);
      }
    }

    ref_type.elem
  } else if let Type::Path(path_type) = *ty {
    let segments_without_generics = path_type
      .path
      .segments
      .iter()
      .map(|segment| segment.ident.to_string())
      .collect::<Vec<_>>()
      .join("::");

    match segments_without_generics.as_str() {
      "sea_orm::JsonValue" | "JsonValue" | "serde_json::Value" => {
        let new_type: Type = parse_quote!(::liquid::model::Value);
        return Box::new(new_type);
      }
      _ => {}
    }

    let transformed_segments = path_type.path.segments.into_iter().map(|segment| {
      let transformed_args = match segment.arguments {
        syn::PathArguments::None => syn::PathArguments::None,
        syn::PathArguments::AngleBracketed(args) => {
          syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
            args: args
              .args
              .into_iter()
              .map(|arg| match arg {
                syn::GenericArgument::Type(arg_type) => {
                  syn::GenericArgument::Type(*eliminate_references(Box::new(arg_type)))
                }
                _ => arg,
              })
              .collect(),
            colon2_token: args.colon2_token,
            gt_token: args.gt_token,
            lt_token: args.lt_token,
          })
        }
        syn::PathArguments::Parenthesized(args) => {
          syn::PathArguments::Parenthesized(syn::ParenthesizedGenericArguments {
            paren_token: args.paren_token,
            output: args.output,
            inputs: args
              .inputs
              .into_iter()
              .map(|arg| *eliminate_references(Box::new(arg)))
              .collect(),
          })
        }
      };
      PathSegment {
        ident: segment.ident,
        arguments: transformed_args,
      }
    });
    Box::new(Type::Path(TypePath {
      qself: path_type.qself,
      path: syn::Path {
        leading_colon: path_type.path.leading_colon,
        segments: transformed_segments.collect(),
      },
    }))
  } else {
    ty
  }
}

pub fn extract_first_generic_arg(path_type: &syn::TypePath) -> Option<Box<Type>> {
  if let Some(last_segment) = path_type.path.segments.last() {
    if let PathArguments::AngleBracketed(last_segment_args) = &last_segment.arguments {
      let value_arg = last_segment_args.args.first().unwrap();
      if let GenericArgument::Type(value_type) = value_arg {
        return Some(Box::new(value_type.clone()));
      }
    }
  }
  None
}

pub fn get_drop_result_generic_arg(ty: Box<Type>) -> Box<Type> {
  let deref_type = eliminate_references(ty);

  if let Type::Path(path_type) = *deref_type.clone() {
    if path_type.path.segments.len() == 1 {
      let segment = path_type.path.segments.first().unwrap();
      if segment.ident == "Result" || segment.ident == "Option" {
        if let Some(value) = extract_first_generic_arg(&path_type) {
          return get_drop_result_generic_arg(value);
        }
      }
    }
  }

  deref_type
}

pub fn build_generic_args<'a>(params: impl Iterator<Item = &'a GenericParam>) -> PathArguments {
  let generics: Punctuated<GenericArgument, Comma> = params
    .filter_map(|param| match param {
      GenericParam::Type(type_param) => {
        let ident = &type_param.ident;
        let type_param_only: GenericArgument = parse_quote!(#ident);
        Some(type_param_only)
      }
      GenericParam::Lifetime(lifetime_param) => {
        Some(GenericArgument::Lifetime(lifetime_param.lifetime.clone()))
      }
      GenericParam::Const(_) => None,
    })
    .collect();

  if generics.is_empty() {
    PathArguments::None
  } else {
    PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
      args: generics,
      colon2_token: Default::default(),
      lt_token: Default::default(),
      gt_token: Default::default(),
    })
  }
}
