mod associations;

extern crate proc_macro;
use associations::{eval_association_macro, AssociationType, TargetType};
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn belongs_to_related(args: TokenStream, input: TokenStream) -> TokenStream {
  eval_association_macro(
    AssociationType::Related,
    TargetType::OneRequired,
    args,
    input,
  )
}

#[proc_macro_attribute]
pub fn belongs_to_linked(args: TokenStream, input: TokenStream) -> TokenStream {
  eval_association_macro(
    AssociationType::Linked,
    TargetType::OneRequired,
    args,
    input,
  )
}

#[proc_macro_attribute]
pub fn has_one_related(args: TokenStream, input: TokenStream) -> TokenStream {
  eval_association_macro(
    AssociationType::Related,
    TargetType::OneOptional,
    args,
    input,
  )
}

#[proc_macro_attribute]
pub fn has_one_linked(args: TokenStream, input: TokenStream) -> TokenStream {
  eval_association_macro(
    AssociationType::Linked,
    TargetType::OneOptional,
    args,
    input,
  )
}

#[proc_macro_attribute]
pub fn has_many_related(args: TokenStream, input: TokenStream) -> TokenStream {
  eval_association_macro(AssociationType::Related, TargetType::Many, args, input)
}

#[proc_macro_attribute]
pub fn has_many_linked(args: TokenStream, input: TokenStream) -> TokenStream {
  eval_association_macro(AssociationType::Linked, TargetType::Many, args, input)
}
