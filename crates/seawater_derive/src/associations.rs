use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
  parse::Parse, parse_macro_input, parse_quote, punctuated::Punctuated, Error, Ident, ImplItem,
  ItemImpl, Path, Token,
};

pub enum AssociationType {
  Related,
  Linked,
}

pub enum TargetType {
  OneOptional,
  OneRequired,
  Many,
}

struct RelatedAssociationMacroArgs {
  name: Ident,
  to: Path,
  inverse: Option<Ident>,
}

struct LinkedAssociationMacroArgs {
  name: Ident,
  to: Path,
  link: Path,
  inverse: Option<Ident>,
}

trait AssociationMacroArgs {
  fn get_name(&self) -> &Ident;
  fn get_to(&self) -> &Path;
  fn get_inverse(&self) -> Option<&Ident>;
  fn get_link(&self) -> Option<&Path>;
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
}

fn start_parsing_args<'a>(
  vars_iter: &mut syn::punctuated::Iter<'a, Path>,
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

fn finish_parsing_args<'a>(
  vars_iter: &mut syn::punctuated::Iter<'a, Path>,
  input: &'a syn::parse::ParseBuffer,
) -> Result<Option<&'a Ident>, Error> {
  let inverse = vars_iter
    .next()
    .map(|path| {
      path
        .get_ident()
        .ok_or_else(|| Error::new(input.span(), "Not a valid identifier"))
    })
    .transpose()?;
  if vars_iter.next().is_some() {
    return Err(Error::new(
      input.span(),
      "Unexpected parameter for association macro",
    ));
  }
  Ok(inverse)
}

impl Parse for RelatedAssociationMacroArgs {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let vars = Punctuated::<Path, Token![,]>::parse_terminated(input)?;
    let mut vars_iter = vars.iter();

    let (name, to) = start_parsing_args(&mut vars_iter, input)?;
    let inverse = finish_parsing_args(&mut vars_iter, input)?;

    Ok(RelatedAssociationMacroArgs {
      name: name.to_owned(),
      to: to.to_owned(),
      inverse: inverse.map(|path| path.to_owned()),
    })
  }
}

impl Parse for LinkedAssociationMacroArgs {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let vars = Punctuated::<Path, Token![,]>::parse_terminated(input)?;
    let mut vars_iter = vars.iter();

    let (name, to) = start_parsing_args(&mut vars_iter, input)?;
    let link = vars_iter
      .next()
      .ok_or_else(|| Error::new(input.span(), "Link expected"))?;
    let inverse = finish_parsing_args(&mut vars_iter, input)?;

    Ok(LinkedAssociationMacroArgs {
      name: name.to_owned(),
      to: to.to_owned(),
      link: link.to_owned(),
      inverse: inverse.map(|path| path.to_owned()),
    })
  }
}

trait AssociationMacro {
  fn preloader_constructor(&self) -> ImplItem;
  fn get_name(&self) -> &Ident;
  fn get_to(&self) -> &Path;
  fn get_target_type(&self) -> &TargetType;
  fn get_inverse(&self) -> Option<&Ident>;
  fn loader_result_type(&self) -> Path;

  fn target_path(&self) -> Path {
    let to_drop = self.get_to();

    match self.get_target_type() {
      TargetType::OneOptional | TargetType::OneRequired => to_drop.clone(),
      TargetType::Many => parse_quote!(Vec<::lazy_liquid_value_view::ArcValueView<#to_drop>>),
    }
  }

  fn imperative_preloader(&self) -> ImplItem {
    let ident = Ident::new(
      format!("preload_{}", self.get_name()).as_str(),
      self.get_name().span(),
    );
    let preloader_ident = self.preloader_ident();
    let target_path = self.target_path();

    parse_quote!(
      pub async fn #ident(
        context: <Self as seawater::ContextContainer>::Context,
        drops: &[&Self],
      ) -> Result<::seawater::preloaders::PreloaderResult<<<<<Self as ::seawater::ModelBackedDrop>::Model as ::sea_orm::ModelTrait>::Entity as ::sea_orm::EntityTrait>::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType, #target_path>, ::seawater::DropError> {
        use ::seawater::preloaders::Preloader;
        use ::seawater::Context;

        let preloader = Self::#preloader_ident(context.clone());
        preloader.preload(context.db(), drops).await
      }
    )
  }

  fn once_cell_getter(&self) -> ImplItem {
    let name = self.get_name();
    let target_path = self.target_path();
    let once_cell_getter_ident = self.once_cell_getter_ident();

    parse_quote!(
      fn #once_cell_getter_ident(cache: &<Self as ::lazy_liquid_value_view::LiquidDrop>::Cache) -> &::once_cell::race::OnceBox<::lazy_liquid_value_view::DropResult<#target_path>> {
        &cache.#name
      }
    )
  }

  fn generate_items(&self) -> Vec<ImplItem> {
    vec![
      self.once_cell_getter(),
      self.preloader_constructor(),
      self.field_getter(),
      self.imperative_preloader(),
    ]
  }

  fn preloader_ident(&self) -> Ident {
    let name = self.get_name();
    Ident::new(format!("{}_preloader", name).as_str(), name.span())
  }

  fn once_cell_getter_ident(&self) -> Ident {
    let name = self.get_name();
    Ident::new(format!("get_{}_once_cell", name).as_str(), name.span())
  }

  fn inverse_once_cell_getter_ident(&self) -> Option<Ident> {
    self.get_inverse().map(|name| {
      Ident::new(
        format!("get_{}_inverse_once_cell", name).as_str(),
        name.span(),
      )
    })
  }

  fn field_getter(&self) -> ImplItem {
    let preloader_ident = self.preloader_ident();
    let name = self.get_name();
    let target_path = self.target_path();

    parse_quote!(
      pub async fn #name(&self) -> Result<#target_path, ::seawater::DropError> {
        use ::seawater::preloaders::Preloader;
        use ::seawater::Context;

        Self::#preloader_ident(self.context.clone())
          .expect_single(self.context.db(), self)
          .await
      }
    )
  }

  fn loader_result_to_drops<'a>(&'a self) -> Box<dyn ToTokens + 'a> {
    let to_drop = self.get_to();
    let loader_result_type = self.loader_result_type();

    Box::new(quote!(
      |result: Option<&#loader_result_type>, from_drop: &Self| {
        result.map(|result| {
          Ok(result.models.iter().map(|model| <#to_drop>::new(model.clone(), from_drop.context.clone())).collect())
        }).unwrap_or_else(|| Ok(vec![]))
      }
    ))
  }

  fn drops_to_value<'a>(&'a self) -> Box<dyn ToTokens + 'a> {
    let to_drop = self.get_to();

    match self.get_target_type() {
      TargetType::OneOptional => Box::new(quote!(|drops: Vec<::std::sync::Arc<#to_drop>>| {
        if drops.len() == 1 {
          Ok(drops[0].clone().into())
        } else {
          Ok(None::<#to_drop>.into())
        }
      })),
      TargetType::OneRequired => Box::new(quote!(|drops: Vec<::std::sync::Arc<#to_drop>>| {
        if drops.len() == 1 {
          Ok(drops[0].clone().into())
        } else {
          Err(::seawater::DropError::ExpectedEntityNotFound(format!(
            "Expected one model, but there are {}",
            drops.len()
          )))
        }
      })),
      TargetType::Many => Box::new(quote!(|drops: Vec<::std::sync::Arc<#to_drop>>| {
        Ok(drops.into_iter().map(|drop| ::lazy_liquid_value_view::ArcValueView(drop)).collect::<Vec<_>>().into())
      })),
    }
  }
}

struct RelatedAssociationMacro {
  name: Ident,
  to: Path,
  target_type: TargetType,
  inverse: Option<Ident>,
}

impl AssociationMacro for RelatedAssociationMacro {
  fn get_name(&self) -> &Ident {
    &self.name
  }

  fn get_target_type(&self) -> &TargetType {
    &self.target_type
  }

  fn get_to(&self) -> &Path {
    &self.to
  }

  fn get_inverse(&self) -> Option<&Ident> {
    self.inverse.as_ref()
  }

  fn loader_result_type(&self) -> Path {
    let to_drop = self.get_to();

    parse_quote!(
      ::intercode_graphql::loaders::EntityRelationLoaderResult<
        <<Self as ::seawater::ModelBackedDrop>::Model as ::sea_orm::ModelTrait>::Entity,
        <<#to_drop as ::seawater::ModelBackedDrop>::Model as ::sea_orm::ModelTrait>::Entity,
      >
    )
  }

  fn preloader_constructor(&self) -> ImplItem {
    let preloader_ident = self.preloader_ident();
    let to_drop = self.get_to();
    let target_path = self.target_path();
    let loader_result_to_drops = self.loader_result_to_drops();
    let drops_to_value = self.drops_to_value();
    let once_cell_getter_ident = self.once_cell_getter_ident();

    let with_inverse_once_cell_getter = self.inverse_once_cell_getter_ident().map(|ident| {
      quote!(
        .with_inverse_once_cell_getter(Self::#ident)
      )
    });

    parse_quote!(
      pub fn #preloader_ident(
        context: <Self as seawater::ContextContainer>::Context
      ) -> <::seawater::preloaders::EntityRelationPreloaderBuilder<Self, #to_drop, #target_path, <Self as seawater::ContextContainer>::Context> as ::seawater::preloaders::PreloaderBuilder>::Preloader {
        use ::seawater::preloaders::PreloaderBuilder;
        use ::seawater::ModelBackedDrop;
        use ::lazy_liquid_value_view::LiquidDropWithID;

        Self::relation_preloader::<#to_drop, #target_path>(
          <<<Self as ::seawater::ModelBackedDrop>::Model as ::sea_orm::ModelTrait>::Entity as ::sea_orm::EntityTrait>::PrimaryKey::Id,
        )
          .with_context(context)
          .with_id_getter(|drop: &Self| drop.id())
          .with_loader_result_to_drops(#loader_result_to_drops)
          .with_drops_to_value(#drops_to_value)
          .with_once_cell_getter(Self::#once_cell_getter_ident)
          #with_inverse_once_cell_getter
          .finalize()
          .unwrap()
      }
    )
  }
}

struct LinkedAssociationMacro {
  name: Ident,
  to: Path,
  link: Path,
  target_type: TargetType,
  inverse: Option<Ident>,
}

impl AssociationMacro for LinkedAssociationMacro {
  fn get_name(&self) -> &Ident {
    &self.name
  }

  fn get_to(&self) -> &Path {
    &self.to
  }

  fn get_target_type(&self) -> &TargetType {
    &self.target_type
  }

  fn get_inverse(&self) -> Option<&Ident> {
    self.inverse.as_ref()
  }

  fn loader_result_type(&self) -> Path {
    let to_drop = self.get_to();

    parse_quote!(
      ::intercode_graphql::loaders::EntityLinkLoaderResult<
        <<Self as ::seawater::ModelBackedDrop>::Model as ::sea_orm::ModelTrait>::Entity,
        <<#to_drop as ::seawater::ModelBackedDrop>::Model as ::sea_orm::ModelTrait>::Entity,
      >
    )
  }

  fn preloader_constructor(&self) -> ImplItem {
    let preloader_ident = self.preloader_ident();
    let to_drop = self.get_to();
    let target_path = self.target_path();
    let loader_result_to_drops = self.loader_result_to_drops();
    let drops_to_value = self.drops_to_value();
    let link = &self.link;
    let once_cell_getter_ident = self.once_cell_getter_ident();

    let with_inverse_once_cell_getter = self.inverse_once_cell_getter_ident().map(|ident| {
      quote!(
        .with_inverse_once_cell_getter(Self::#ident)
      )
    });

    parse_quote!(
      pub fn #preloader_ident(
        context: <Self as seawater::ContextContainer>::Context
      ) -> <::seawater::preloaders::EntityLinkPreloaderBuilder<Self, #to_drop, #target_path, <Self as seawater::ContextContainer>::Context> as ::seawater::preloaders::PreloaderBuilder>::Preloader
      {
        use ::seawater::preloaders::PreloaderBuilder;
        use ::seawater::ModelBackedDrop;
        use ::lazy_liquid_value_view::LiquidDropWithID;

        Self::link_preloader::<#to_drop, #target_path>(
          #link,
          <<<Self as ::seawater::ModelBackedDrop>::Model as ::sea_orm::ModelTrait>::Entity as ::sea_orm::EntityTrait>::PrimaryKey::Id,
        )
        .with_context(context)
        .with_id_getter(|drop: &Self| drop.id())
        .with_loader_result_to_drops(#loader_result_to_drops)
        .with_drops_to_value(#drops_to_value)
        .with_once_cell_getter(Self::#once_cell_getter_ident)
        #with_inverse_once_cell_getter
        .finalize()
        .unwrap()
      }
    )
  }
}

pub fn eval_association_macro(
  association_type: AssociationType,
  target_type: TargetType,
  args: TokenStream,
  input: TokenStream,
) -> TokenStream {
  let args: Box<dyn AssociationMacroArgs> = match association_type {
    AssociationType::Related => Box::new(parse_macro_input!(args as RelatedAssociationMacroArgs)),
    AssociationType::Linked => Box::new(parse_macro_input!(args as LinkedAssociationMacroArgs)),
  };
  let mut input = parse_macro_input!(input as ItemImpl);

  let association: Box<dyn AssociationMacro> = match association_type {
    AssociationType::Related => Box::new(RelatedAssociationMacro {
      name: args.get_name().to_owned(),
      to: args.get_to().to_owned(),
      target_type,
      inverse: args.get_inverse().map(|inverse| inverse.to_owned()),
    }),
    AssociationType::Linked => {
      if let Some(link) = args.get_link() {
        Box::new(LinkedAssociationMacro {
          link: link.to_owned(),
          name: args.get_name().to_owned(),
          target_type,
          to: args.get_to().to_owned(),
          inverse: args.get_inverse().map(|inverse| inverse.to_owned()),
        })
      } else {
        panic!("Linked associations require a link");
      }
    }
  };

  let mut items = association.generate_items();
  input.items.append(&mut items);

  quote!(#input).into()
}
