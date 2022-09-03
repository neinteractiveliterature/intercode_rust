extern crate proc_macro;
use lazy_value_view::eval_lazy_value_view_macro;
use liquid_drop_impl::eval_liquid_drop_impl_macro;
use liquid_drop_struct::eval_liquid_drop_struct_macro;
use proc_macro::TokenStream;

mod drop_getter_method;
mod drop_method_attribute;
mod helpers;
mod lazy_value_view;
mod liquid_drop_impl;
mod liquid_drop_struct;

#[proc_macro_attribute]
pub fn lazy_value_view(args: TokenStream, input: TokenStream) -> TokenStream {
  eval_lazy_value_view_macro(args, input)
}

#[proc_macro_attribute]
pub fn liquid_drop_struct(args: TokenStream, input: TokenStream) -> TokenStream {
  eval_liquid_drop_struct_macro(args, input)
}

#[proc_macro_attribute]
pub fn liquid_drop_impl(args: TokenStream, input: TokenStream) -> TokenStream {
  eval_liquid_drop_impl_macro(args, input)
}
