use darling::{ast, FromDeriveInput, FromField, FromMeta, FromVariant};
use quote::{format_ident, quote};

use crate::util::{Args, Field, Map};

#[derive(FromDeriveInput)]
#[darling(attributes(transform))]
pub struct Transform {
	pub data: ast::Data<TransformVariant, TransformFieldWrapper>,

	#[darling(multiple)]
	pub func: Vec<syn::Expr>,

	#[darling(default)]
	pub custom: Map<syn::Path, Option<Args>>,

	#[darling(default)]
	pub custom_async: Map<syn::Path, Option<Args>>,
}

pub struct TransformOptions {
	pub func: Vec<syn::Expr>,
	pub custom: Map<syn::Path, Option<Args>>,
	pub custom_async: Map<syn::Path, Option<Args>>,
}

#[derive(Debug, FromVariant)]
pub struct TransformVariant {
	pub ident: syn::Ident,
	pub fields: ast::Fields<TransformFieldWrapper>,
}

#[derive(Debug, FromField)]
#[darling(attributes(transform))]
pub struct TransformFieldWrapper {
	pub ident: Option<syn::Ident>,
	pub ty: syn::Type,

	#[darling(multiple)]
	func: Vec<syn::Expr>,

	#[darling(default)]
	custom: Map<syn::Path, Option<Args>>,

	#[darling(default)]
	pub custom_async: Map<syn::Path, Option<Args>>,

	#[darling(default)]
	inner: Option<Box<TransformField>>,

	dive: darling::util::Flag,

	#[darling(flatten)]
	builtin: Map<syn::Path, Option<Args>>,
}

#[derive(Debug, FromMeta)]
struct TransformField {
	#[darling(multiple)]
	func: Vec<syn::Expr>,

	#[darling(default)]
	custom: Map<syn::Path, Option<Args>>,

	#[darling(default)]
	custom_async: Map<syn::Path, Option<Args>>,

	#[darling(default)]
	inner: Option<Box<TransformField>>,

	dive: darling::util::Flag,

	#[darling(flatten)]
	builtin: Map<syn::Path, Option<Args>>,
}

impl TransformFieldWrapper {
	fn into_inner(self) -> TransformField {
		TransformField {
			func: self.func,
			custom: self.custom,
			custom_async: self.custom_async,
			inner: self.inner,
			dive: self.dive,
			builtin: self.builtin,
		}
	}
}

impl TransformFieldWrapper {
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

impl TransformOptions {
	pub fn into_token_stream(
		self,
		crate_name: &syn::Path,
		ty: &syn::Type,
	) -> proc_macro2::TokenStream {
		TransformField {
			func: self.func,
			custom: self.custom,
			custom_async: self.custom_async,
			inner: None,
			dive: darling::util::Flag::default(),
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

impl TransformField {
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
				#crate_name::Transformer::transform(
					&#crate_name::options::transformer::#path::Transformer::new() #args,
					&(),
					#field
				);
			});
		}

		if let Some(inner) = &mut self.inner {
			let inner = inner.to_token_stream(crate_name, field, ty, false);

			tokens.extend(quote! {
				for mut #field in #crate_name::AsMutSlice::as_mut_slice(#field) {
					#inner
				}
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
				#crate_name::Transformer::transform(
					&transformer::#path::new() #args,
					ctx,
					#field
				);
			});
		}

		for (path, args) in self.custom_async.iter() {
			tokens.extend(quote! {
				#crate_name::AsyncTransformer::transform_async(
					&transformer::#path::new() #args,
					ctx,
					#field
				).await;
			});
		}

		if self.dive.is_present() {
			tokens.extend(quote! {
				#crate_name::Transform::transform(#field, ctx);
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
