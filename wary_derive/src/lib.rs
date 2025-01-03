#![warn(clippy::pedantic, clippy::print_stdout)]
#![allow(clippy::too_many_lines)]

use darling::FromDeriveInput;
use emit::Emit;

mod attr;
mod emit;
mod modify;
mod util;
mod validate;

#[proc_macro_derive(Wary, attributes(validate, modify, wary, serde))]
pub fn wary(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = syn::parse_macro_input!(input as syn::DeriveInput);

	match Emit::from_derive_input(&input) {
		Ok(validate) => validate.into_token_stream(),
		Err(e) => e.write_errors(),
	}
	.into()
}
