use syn::{
  parse_quote, punctuated::Punctuated, token::Comma, Error, GenericArgument, GenericParam,
  PathArguments, PathSegment, Type, TypeGroup, TypeParamBound, TypePath,
};

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

pub fn extract_generic_args(path_type: &syn::TypePath) -> Vec<Type> {
  if let Some(last_segment) = path_type.path.segments.last() {
    if let PathArguments::AngleBracketed(last_segment_args) = &last_segment.arguments {
      return last_segment_args
        .args
        .iter()
        .filter_map(|arg| {
          if let GenericArgument::Type(value_type) = arg {
            Some(value_type.clone())
          } else {
            None
          }
        })
        .collect();
    }
  }

  vec![]
}

pub fn extract_first_generic_arg(path_type: &syn::TypePath) -> Option<Type> {
  extract_generic_args(path_type).first().cloned()
}

pub fn path_without_generics(path: &syn::Path) -> syn::Path {
  let segments = path.segments.iter().map(|segment| PathSegment {
    ident: segment.ident.clone(),
    arguments: PathArguments::None,
  });

  syn::Path {
    leading_colon: None,
    segments: segments.collect(),
  }
}

pub fn get_drop_result_generic_arg(ty: Box<Type>) -> Box<Type> {
  let deref_type = eliminate_references(ty);

  if let Type::Path(path_type) = *deref_type.clone() {
    let normalized_path = path_without_generics(&path_type.path);
    if normalized_path == parse_quote!(DropRef)
      || normalized_path == parse_quote!(seawater::DropRef)
    {
      if let Some(value) = extract_first_generic_arg(&path_type) {
        return get_drop_result_generic_arg(Box::new(value));
      }
    } else if normalized_path == parse_quote!(Option) {
      if let Some(value) = extract_first_generic_arg(&path_type) {
        let non_optional_type = get_drop_result_generic_arg(Box::new(value));
        return parse_quote!(::seawater::OptionalValueView<#non_optional_type>);
      }
    } else if normalized_path == parse_quote!(Result) {
      let generics = extract_generic_args(&path_type);
      if generics.len() != 2 {
        panic!("Result requires exactly 2 type arguments");
      }
      let non_result_type =
        get_drop_result_generic_arg(Box::new(generics.first().cloned().unwrap()));
      let error_type = generics.last().unwrap();
      return parse_quote!(::seawater::ResultValueView<#non_result_type, #error_type>);
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
