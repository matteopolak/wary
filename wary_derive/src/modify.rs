use darling::{FromDeriveInput, FromField, FromMeta, FromVariant, ast};
use quote::{format_ident, quote};

use crate::util::{Args, Field, Map};

#[derive(FromDeriveInput)]
#[darling(attributes(modify))]
pub struct Modify {
	pub data: ast::Data<ModifyVariant, ModifyFieldWrapper>,

	#[darling(multiple)]
	pub func: Vec<syn::Expr>,

	#[darling(default)]
	pub custom: Map<syn::Path, Option<Args>>,
}

pub struct ModifyOptions {
	pub func: Vec<syn::Expr>,
	pub custom: Map<syn::Path, Option<Args>>,
}

#[derive(Debug, FromVariant)]
pub struct ModifyVariant {
	pub ident: syn::Ident,
	pub fields: ast::Fields<ModifyFieldWrapper>,
}

#[derive(Debug, FromField)]
#[darling(attributes(modify))]
pub struct ModifyFieldWrapper {
	pub ident: Option<syn::Ident>,
	pub ty: syn::Type,

	#[darling(multiple)]
	func: Vec<syn::Expr>,

	#[darling(default)]
	custom: Map<syn::Path, Option<Args>>,

	#[darling(default)]
	inner: Option<Box<ModifyField>>,

	#[darling(flatten)]
	builtin: Map<syn::Path, Option<Args>>,
}

#[derive(Debug, FromMeta)]
struct ModifyField {
	#[darling(multiple)]
	func: Vec<syn::Expr>,

	#[darling(default)]
	custom: Map<syn::Path, Option<Args>>,

	#[darling(default)]
	inner: Option<Box<ModifyField>>,

	#[darling(flatten)]
	builtin: Map<syn::Path, Option<Args>>,
}

impl ModifyFieldWrapper {
	fn into_inner(self) -> ModifyField {
		ModifyField {
			func: self.func,
			custom: self.custom,
			inner: self.inner,
			builtin: self.builtin,
		}
	}
}

impl ModifyFieldWrapper {
	pub fn into_token_stream(
		self,
		crate_name: &syn::Path,
		field: &Field,
	) -> proc_macro2::TokenStream {
		let ty = self.ty.clone();
		self
			.into_inner()
			.to_token_stream(crate_name, field, &ty, true)
	}
}

impl ModifyOptions {
	pub fn into_token_stream(
		self,
		crate_name: &syn::Path,
		ty: &syn::Type,
	) -> proc_macro2::TokenStream {
		ModifyField {
			func: self.func,
			custom: self.custom,
			inner: None,
			builtin: Map::default(),
		}
		.to_token_stream(
			crate_name,
			&Field::new_ident(format_ident!("self")),
			ty,
			true,
		)
	}
}

impl ModifyField {
	#[allow(clippy::wrong_self_convention)]
	fn to_token_stream(
		&mut self,
		crate_name: &syn::Path,
		field: &Field,
		ty: &syn::Type,
		top: bool,
	) -> proc_macro2::TokenStream {
		let mut tokens = proc_macro2::TokenStream::new();
		let option_path = crate::attr::extract_option_path(ty);

		for (path, args) in self.builtin.iter_mut() {
			tokens.extend(quote! {
				#crate_name::Modifier::modify(
					&#crate_name::options::modifier::#path::Modifier::new() #args,
					&(),
					#field
				);
			});
		}

		if let Some(inner) = &mut self.inner {
			let inner = inner.to_token_stream(crate_name, field, ty, false);

			tokens.extend(quote! {
				#crate_name::Modifier::modify(
					&#crate_name::options::modifier::inner::Modifier::new(|#field| {
						#inner
					}),
					&(),
					#field
				);
			});
		}

		for func in &self.func {
			tokens.extend(quote! {
				{
					let _: () = (#func)(ctx, #field);
				}
			});
		}

		for (path, args) in self.custom.iter() {
			tokens.extend(quote! {
				#crate_name::Modifier::modify(
					&#path::new() #args,
					ctx,
					#field
				);
			});
		}

		if top {
			if let Some(ref option_path) = option_path {
				return quote! {
					if let #option_path ::Some(#field) = #field {
						#tokens
					}
				};
			}
		}

		tokens
	}
}
