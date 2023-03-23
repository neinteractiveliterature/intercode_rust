use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse, parse_macro_input, parse_quote, Ident, ImplItem, ItemImpl, LitStr, Path};

use crate::attrs::{AssociationMacroArgs, LinkedAssociationMacroArgs, RelatedAssociationMacroArgs};

pub enum AssociationType {
  Related,
  Linked,
}

pub enum TargetType {
  OneOptional,
  OneRequired,
  Many,
}

trait AssociationMacro {
  fn preloader_constructor(&self) -> ImplItem;
  fn get_name(&self) -> &Ident;
  fn get_to(&self) -> &Path;
  fn get_target_type(&self) -> &TargetType;
  fn get_inverse(&self) -> Option<&Ident>;
  fn get_eager_load_associations(&self) -> &[Ident];
  fn should_serialize(&self) -> bool;
  fn loader_result_type(&self) -> Path;

  fn target_path(&self) -> Path {
    let to_drop = self.get_to();

    match self.get_target_type() {
      TargetType::OneOptional | TargetType::OneRequired => {
        parse_quote!(#to_drop)
      }
      TargetType::Many => parse_quote!(Vec<#to_drop>),
    }
  }

  fn imperative_preloader(&self) -> ImplItem {
    let ident = Ident::new(
      format!("preload_{}", self.get_name()).as_str(),
      self.get_name().span(),
    );
    let preloader_ident = self.preloader_ident();
    let target_path = self.target_path();
    let eager_load_action = self.eager_load_action();
    let ident_str = LitStr::new(ident.to_string().as_str(), ident.span());
    let get_preloaded_drops = match self.get_target_type() {
      TargetType::Many => Box::new(quote!(
        let preloaded_drops = preloader_result.all_values_flat().collect::<Vec<_>>();
      )),
      TargetType::OneRequired => Box::new(quote!(
        let preloaded_drops = preloader_result.all_values_unwrapped().collect::<Vec<_>>();
      )),
      TargetType::OneOptional => Box::new(quote!(
        let preloaded_drops = preloader_result.all_values().filter_map(|v| v.get_inner_cloned()).collect::<Vec<_>>();
      )),
    };

    parse_quote!(
      pub fn #ident<'a>(
        context: <Self as ::seawater::LiquidDrop>::Context,
        drops: &'a [::seawater::DropRef<Self>],
      ) -> ::futures::future::BoxFuture<'a, Result<::seawater::preloaders::PreloaderResult<<<<<Self as ::seawater::ModelBackedDrop>::Model as ::sea_orm::ModelTrait>::Entity as ::sea_orm::EntityTrait>::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType, #target_path>, ::seawater::DropError>> {
        use ::futures::FutureExt;

        async move {
          use ::seawater::preloaders::Preloader;
          use ::seawater::Context;
          use ::tracing::log::info;

          info!(
            "{}.{}: eager-loading {} {}",
            ::seawater::pretty_type_name::<Self>(),
            #ident_str,
            drops.len(),
            ::seawater::pretty_type_name::<#target_path>()
          );

          let preloader = Self::#preloader_ident(context.clone());
          let preloader_result = preloader.preload(context.db(), drops).await?;
          #get_preloaded_drops
          #eager_load_action
          Ok(preloader_result)
        }.boxed()
      }
    )
  }

  fn once_cell_getter(&self) -> ImplItem {
    let name = self.get_name();
    let target_path = self.target_path();
    let once_cell_getter_ident = self.once_cell_getter_ident();

    parse_quote!(
      fn #once_cell_getter_ident(cache: &<Self as ::seawater::LiquidDrop>::Cache) -> &::once_cell::race::OnceBox<::seawater::DropResult<#target_path>> {
        &cache.#name
      }
    )
  }

  fn generate_items(&self) -> Vec<ImplItem> {
    vec![
      Some(self.once_cell_getter()),
      Some(self.preloader_constructor()),
      Some(self.field_getter()),
      Some(self.imperative_preloader()),
    ]
    .into_iter()
    .flatten()
    .collect()
  }

  fn preloader_ident(&self) -> Ident {
    let name = self.get_name();
    Ident::new(format!("{}_preloader", name).as_str(), name.span())
  }

  fn once_cell_getter_ident(&self) -> Ident {
    let name = self.get_name();
    Ident::new(format!("get_{}_once_cell", name).as_str(), name.span())
  }

  fn inverse_once_cell_getter(&self) -> Box<dyn ToTokens> {
    Box::new(
      self
        .get_inverse()
        .map(|name| {
          let to_drop_ident = self.get_to();
          quote!(
            Some(Box::pin(|drop_cache: &<#to_drop_ident as ::seawater::LiquidDrop>::Cache| {
              &drop_cache.#name
            }))
          )
        })
        .unwrap_or(quote!(None)),
    )
  }

  fn field_getter(&self) -> ImplItem {
    let preloader_ident = self.preloader_ident();
    let name = self.get_name();
    let target_path = self.target_path();
    let serialize_attr = if self.should_serialize() {
      Some(quote!(#[liquid_drop(serialize_value = true)]))
    } else {
      None
    };
    let ident_str = LitStr::new(name.to_string().as_str(), name.span());
    let result_to_vec = match self.get_target_type() {
      TargetType::Many => Box::new(quote!(
        let preloaded_drops = drop.clone();
      )),
      TargetType::OneRequired | &TargetType::OneOptional => Box::new(quote!(
        let preloaded_drops = vec![drop.clone()];
      )),
    };
    let eager_load_action = self.eager_load_action();

    parse_quote!(
      #serialize_attr
      pub async fn #name(&self) -> Result<#target_path, ::seawater::DropError> {
        use ::seawater::LiquidDrop;
        use ::seawater::preloaders::Preloader;
        use ::seawater::Context;
        use ::tracing::log::info;

        info!(
          "{}.{}: lazy-loading 1 {}",
          ::seawater::pretty_type_name::<Self>(),
          #ident_str,
          ::seawater::pretty_type_name::<#target_path>()
        );

        let drop_ref = self.context.with_drop_store(|store| {
          store.store(self.clone())
        });
        let drop = Self::#preloader_ident(self.context.clone())
          .expect_single(self.context.db(), drop_ref)
          .await?;
        let context = self.context.clone();
        #result_to_vec
        #eager_load_action
        Ok(drop)
      }
    )
  }

  fn eager_load_action<'a>(&'a self) -> Box<dyn ToTokens + 'a> {
    let eager_load_associations = self.get_eager_load_associations();
    if eager_load_associations.is_empty() {
      return Box::new(quote!());
    }

    let to_drop = self.get_to();

    let eager_loads = self
      .get_eager_load_associations()
      .iter()
      .map(|association_name| {
        let imperative_preload_ident = Ident::new(
          format!("preload_{}", association_name).as_str(),
          association_name.span(),
        );
        quote!(
          #to_drop::#imperative_preload_ident(context.clone(), &preloaded_drop_refs)
        )
      });

    Box::new(quote!(
      let preloaded_drop_refs = context.with_drop_store(|store| store.store_all(preloaded_drops));

      ::futures::try_join!(
        #(#eager_loads,)*
      )?;
    ))
  }

  fn loader_result_to_drops<'a>(&'a self) -> Box<dyn ToTokens + 'a> {
    let to_drop = self.get_to();
    let loader_result_type = self.loader_result_type();

    Box::new(quote!(
      |result: Option<#loader_result_type>, from_drop: ::seawater::DropRef<Self>| {
        result.map(|result| {
          Ok(result.models.into_iter().map(|model| #to_drop::new(model, from_drop.fetch().context.clone())).collect())
        }).unwrap_or_else(|| Ok(vec![]))
      }
    ))
  }

  fn drops_to_value<'a>(&'a self) -> Box<dyn ToTokens + 'a> {
    let to_drop = self.get_to();

    match self.get_target_type() {
      TargetType::OneOptional => Box::new(quote!(
        |store: &::seawater::DropStore<::seawater::DropStoreID<Self>>,
         drops: Vec<::seawater::DropRef<#to_drop>>| {
        if drops.len() == 1 {
          Ok(drops[0].into())
        } else {
          Ok(None::<#to_drop>.into())
        }
      })),
      TargetType::OneRequired => Box::new(quote!(
        |store: &::seawater::DropStore<::seawater::DropStoreID<Self>>,
         drops: Vec<::seawater::DropRef<#to_drop>>| {
        if drops.len() == 1 {
          Ok(drops[0].into())
        } else {
          Err(::seawater::DropError::ExpectedEntityNotFound(format!(
            "Expected one {}, but there are {}",
            ::seawater::pretty_type_name::<#to_drop>(),
            drops.len()
          )))
        }
      })),
      TargetType::Many => Box::new(quote!(
        |store: &::seawater::DropStore<<<Self as ::seawater::LiquidDrop>::Context as ::seawater::Context>::StoreID>,
         drops: Vec<::seawater::DropRef<#to_drop>>| {
          Ok(store.get_all::<#to_drop>(drops.into_iter().map(|drop| drop.id())).into())
        }
      )),
    }
  }
}

struct RelatedAssociationMacro {
  name: Ident,
  to: Path,
  target_type: TargetType,
  inverse: Option<Ident>,
  eager_load_associations: Vec<Ident>,
  serialize: bool,
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

  fn get_eager_load_associations(&self) -> &[Ident] {
    &self.eager_load_associations
  }

  fn should_serialize(&self) -> bool {
    self.serialize
  }

  fn loader_result_type(&self) -> Path {
    let to_drop = self.get_to();

    parse_quote!(
      ::seawater::loaders::EntityRelationLoaderResult<
        ::seawater::DropEntity<Self>,
        ::seawater::DropEntity<#to_drop>
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
    let inverse_once_cell_getter = self.inverse_once_cell_getter();

    let item = quote!(
      pub fn #preloader_ident(
        context: <Self as ::seawater::LiquidDrop>::Context
      ) -> ::seawater::preloaders::EntityRelationPreloader::<
        ::seawater::DropEntity<Self>,
        ::seawater::DropEntity<#to_drop>,
        ::seawater::DropPrimaryKey<Self>,
        Self,
        #to_drop,
        #target_path,
        <Self as ::seawater::LiquidDrop>::Context,
      > {
        use ::seawater::ModelBackedDrop;
        use ::seawater::LiquidDrop;

        ::seawater::preloaders::EntityRelationPreloader::<
          ::seawater::DropEntity<Self>,
          ::seawater::DropEntity<#to_drop>,
          ::seawater::DropPrimaryKey<Self>,
          Self,
          #to_drop,
          #target_path,
          <Self as ::seawater::LiquidDrop>::Context,
        >::new(
          <<<Self as ::seawater::ModelBackedDrop>::Model as ::sea_orm::ModelTrait>::Entity as ::sea_orm::EntityTrait>::PrimaryKey::Id,
          context,
          #loader_result_to_drops,
          #drops_to_value,
          Self::#once_cell_getter_ident,
          #inverse_once_cell_getter
        )
      }
    );

    parse(item.into()).unwrap()
  }
}

struct LinkedAssociationMacro {
  name: Ident,
  to: Path,
  link: Path,
  target_type: TargetType,
  inverse: Option<Ident>,
  eager_load_associations: Vec<Ident>,
  serialize: bool,
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

  fn get_eager_load_associations(&self) -> &[Ident] {
    &self.eager_load_associations
  }

  fn should_serialize(&self) -> bool {
    self.serialize
  }

  fn loader_result_type(&self) -> Path {
    let to_drop = self.get_to();

    parse_quote!(
      ::seawater::loaders::EntityLinkLoaderResult<
        ::seawater::DropEntity<Self>,
        ::seawater::DropEntity<#to_drop>
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
    let inverse_once_cell_getter = self.inverse_once_cell_getter();

    parse_quote!(
      pub fn #preloader_ident(
        context: <Self as ::seawater::LiquidDrop>::Context
      ) -> ::seawater::preloaders::EntityLinkPreloader::<
        ::seawater::DropEntity<Self>,
        ::seawater::DropEntity<#to_drop>,
        ::seawater::DropPrimaryKey<Self>,
        Self,
        #to_drop,
        #target_path,
        <Self as ::seawater::LiquidDrop>::Context,
      >
      {
        use ::seawater::ModelBackedDrop;
        use ::seawater::LiquidDrop;

        ::seawater::preloaders::EntityLinkPreloader::new(
          <<<Self as ::seawater::ModelBackedDrop>::Model as ::sea_orm::ModelTrait>::Entity as ::sea_orm::EntityTrait>::PrimaryKey::Id,
          #link,
          context,
          #loader_result_to_drops,
          #drops_to_value,
          Self::#once_cell_getter_ident,
          #inverse_once_cell_getter
        )
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
      eager_load_associations: args.get_eager_load_associations().to_vec(),
      serialize: args.should_serialize(),
    }),
    AssociationType::Linked => {
      if let Some(link) = args.get_link() {
        Box::new(LinkedAssociationMacro {
          link: link.to_owned(),
          name: args.get_name().to_owned(),
          target_type,
          to: args.get_to().to_owned(),
          inverse: args.get_inverse().map(|inverse| inverse.to_owned()),
          eager_load_associations: args.get_eager_load_associations().to_vec(),
          serialize: args.should_serialize(),
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
