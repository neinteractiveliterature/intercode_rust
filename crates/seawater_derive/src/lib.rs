mod associations;
mod attrs;

extern crate proc_macro;
use associations::{eval_association_macro, AssociationType, TargetType};
use proc_macro::TokenStream;

use liquid_drop_impl::eval_liquid_drop_impl_macro;
use liquid_drop_struct::eval_liquid_drop_struct_macro;

mod drop_getter_method;
mod drop_method_attribute;
mod helpers;
mod liquid_drop_impl;
mod liquid_drop_struct;

#[proc_macro_attribute]
pub fn liquid_drop_struct(args: TokenStream, input: TokenStream) -> TokenStream {
  eval_liquid_drop_struct_macro(args, input)
}

#[proc_macro_attribute]
pub fn liquid_drop_impl(args: TokenStream, input: TokenStream) -> TokenStream {
  eval_liquid_drop_impl_macro(args, input)
}

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
