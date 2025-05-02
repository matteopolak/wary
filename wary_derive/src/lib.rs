#![warn(clippy::pedantic, clippy::print_stdout)]
#![allow(clippy::too_many_lines, clippy::option_option)]
#![cfg_attr(not(feature = "serde"), allow(unused_variables, unused_mut, dead_code))]

use darling::FromDeriveInput;

mod as_ref;
mod attr;
mod util;
mod wary;

pub(crate) fn default_crate_name() -> syn::Path {
	syn::parse_quote! { ::wary }
}

#[proc_macro_derive(Wary, attributes(validate, transform, wary, serde))]
pub fn wary(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let mut input = syn::parse_macro_input!(input as syn::DeriveInput);

	match wary::emit::Emit::from_derive_input(&mut input) {
		Ok(validate) => validate.into_token_stream(),
		Err(e) => e.write_errors(),
	}
	.into()
}

#[proc_macro_derive(AsRef, attributes(as_ref))]
pub fn as_ref(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = syn::parse_macro_input!(input as syn::DeriveInput);

	match as_ref::emit::AsRef::from_derive_input(&input) {
		Ok(validate) => validate.into_token_stream(),
		Err(e) => e.write_errors(),
	}
	.into()
}
