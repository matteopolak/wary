use darling::{ast, FromDeriveInput, FromField};
use proc_macro2::TokenStream;
use quote::quote;

use crate::util::Field;

#[derive(FromDeriveInput)]
#[darling(attributes(as_ref), supports(struct_any))]
pub struct AsRef {
	pub ident: syn::Ident,
	pub data: ast::Data<(), AsRefField>,

	#[darling(default = "crate::default_crate_name", rename = "crate")]
	crate_name: syn::Path,
}

#[derive(Debug, FromField)]
pub struct AsRefField {
	pub ident: Option<syn::Ident>,
	pub ty: syn::Type,

	skip: darling::util::Flag,
}

impl AsRef {
	pub fn into_token_stream(self) -> TokenStream {
		let crate_name = &self.crate_name;
		let ident = &self.ident;

		let mut tokens = TokenStream::new();
		let s = self.data.take_struct().unwrap();

		for (idx, field) in s.fields.into_iter().enumerate() {
			if field.skip.is_present() {
				continue;
			}

			let ty = field.ty;
			let field = field.ident.map_or_else(
				|| Field::new_index(idx, false),
				|f| Field::new_ident(f, false),
			);

			tokens.extend(quote! {
				impl #crate_name::AsRef<#ty> for #ident {
					fn as_ref(&self) -> &#ty {
						&self.#field
					}
				}
			});
		}

		tokens
	}
}
