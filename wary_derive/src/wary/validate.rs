use darling::{ast, FromDeriveInput, FromField, FromMeta, FromVariant};
use quote::{format_ident, quote, ToTokens};

use crate::{
	attr,
	util::{Args, ArgsRef, Field, Map, Tuple},
};

#[derive(FromDeriveInput)]
#[darling(attributes(validate))]
pub struct Validate {
	pub data: ast::Data<ValidateVariant, ValidateFieldWrapper>,

	#[darling(multiple)]
	pub func: Vec<syn::Expr>,

	#[darling(default)]
	pub or: Tuple<ValidateField>,

	#[darling(default)]
	pub and: Tuple<ValidateField>,

	#[darling(default)]
	pub custom: Map<syn::Path, Option<Args>>,

	#[darling(default)]
	pub custom_async: Map<syn::Path, Option<Args>>,
}

pub struct ValidateOptions {
	pub func: Vec<syn::Expr>,
	pub or: Tuple<ValidateField>,
	pub and: Tuple<ValidateField>,
	pub custom: Map<syn::Path, Option<Args>>,
	pub custom_async: Map<syn::Path, Option<Args>>,
}

#[derive(Debug, FromVariant)]
pub struct ValidateVariant {
	pub ident: syn::Ident,
	pub fields: ast::Fields<ValidateFieldWrapper>,
}

#[derive(Debug, FromMeta)]
pub struct ValidateField {
	#[darling(multiple)]
	func: Vec<syn::Expr>,

	#[darling(default)]
	or: Tuple<ValidateField>,

	#[darling(default)]
	and: Tuple<ValidateField>,

	#[darling(default)]
	custom: Map<syn::Path, Option<Args>>,

	#[darling(default)]
	custom_async: Map<syn::Path, Option<Args>>,

	#[darling(default)]
	inner: Option<Box<ValidateField>>,

	dive: darling::util::Flag,

	#[darling(default)]
	required: Option<Option<Args>>,

	#[darling(flatten)]
	builtin: Map<syn::Path, Option<Args>>,
}

#[derive(Debug, FromField)]
#[darling(attributes(validate))]
pub struct ValidateFieldWrapper {
	pub ident: Option<syn::Ident>,
	pub ty: syn::Type,

	#[darling(multiple)]
	func: Vec<syn::Expr>,

	#[darling(default)]
	or: Tuple<ValidateField>,

	#[darling(default)]
	and: Tuple<ValidateField>,

	#[darling(default)]
	custom: Map<syn::Path, Option<Args>>,

	#[darling(default)]
	pub custom_async: Map<syn::Path, Option<Args>>,

	#[darling(default)]
	inner: Option<Box<ValidateField>>,

	dive: darling::util::Flag,

	#[darling(default)]
	required: Option<Option<Args>>,

	#[darling(flatten)]
	builtin: Map<syn::Path, Option<Args>>,
}

impl ValidateField {
	#[allow(clippy::wrong_self_convention)]
	fn to_token_stream(
		&mut self,
		crate_name: &syn::Path,
		field: &Field,
		ty: &syn::Type,
		top: bool,
	) -> proc_macro2::TokenStream {
		let mut tokens = proc_macro2::TokenStream::new();

		if top {
			if let Some(field_path) = field.path() {
				tokens.extend(quote! {
					let __wary_field = #field_path;
				});
			}
		}

		let error_path = if field.path().is_some() {
			quote! { __wary_parent.append(__wary_field) }
		} else {
			quote! { __wary_parent.clone() }
		};

		let option_path = crate::attr::extract_option_path(ty);

		if option_path.is_none() {
			if let Some(args) = &self.required {
				tokens.extend(quote! {
					if let Err(e) = #crate_name::Rule::validate(
						&#crate_name::options::rule::required::Rule::new() #args,
						&(),
						#field
					) {
						__wary_report.push(#error_path, e);
					};
				});
			}
		}

		for (path, args) in self.builtin.iter_mut() {
			let args_ref = args.as_ref().map(ArgsRef);
			let args: &dyn ToTokens = if path.is_ident("range") {
				&args_ref
			} else if path.is_ident("regex") {
				let key = syn::parse_quote! { pat };

				if let Some(args) = args {
					if let Some(Some(expr)) = args.get(&key) {
						if let Some(s) = attr::extract_str(expr) {
							args.insert(
								key,
								Some(syn::parse_quote! {
									{
										#crate_name::internal::init_regex!(static PAT = #s);
										&PAT
									}
								}),
							);
						}
					}
				}

				args
			} else {
				args
			};

			tokens.extend(quote! {
				if let Err(e) = #crate_name::Rule::validate(
					&#crate_name::options::rule::#path::Rule::new() #args,
					&(),
					#field
				) {
					__wary_report.push(#error_path, e);
				};
			});
		}

		if let Some(inner) = &mut self.inner {
			let inner = inner.to_token_stream(crate_name, field, ty, false);

			tokens.extend(quote! {
				{
					let __wary_parent = #error_path;
					for (__wary_field, #field) in #crate_name::AsSlice::as_slice(#field).iter().enumerate() {
						#inner
					}
				}
			});
		}

		for func in &self.func {
			tokens.extend(quote! {
				{
					let result: ::core::result::Result<(), #crate_name::Error> = (#func)(ctx, #field);
					if let Err(e) = result {
						__wary_report.push(#error_path, e);
					};
				}
			});
		}

		for (path, args) in self.custom.iter() {
			tokens.extend(quote! {
				if let Err(e) = #crate_name::Rule::validate(
					&rule::#path::new() #args,
					ctx,
					#field
				) {
					__wary_report.push(#error_path, e);
				};
			});
		}

		for (path, args) in self.custom_async.iter() {
			tokens.extend(quote! {
				if let Err(e) = #crate_name::AsyncRule::validate_async(
					&rule::#path::new() #args,
					ctx,
					#field
				).await {
					__wary_report.push(#error_path, e);
				};
			});
		}

		let mut and = self.and.0.iter_mut();

		if let Some(and) = and.next() {
			let expand = and.to_token_stream(crate_name, field, ty, false);

			tokens.extend(quote! {
				let __wary_len = __wary_report.len();
				#expand ;
			});
		}

		for and in and {
			let expand = and.to_token_stream(crate_name, field, ty, false);

			tokens.extend(quote! {
				if __wary_report.len() == __wary_len {
					#expand ;
				}
			});
		}

		let mut or = self.or.0.iter_mut();
		let mut or_tokens = proc_macro2::TokenStream::new();

		if let Some(or) = or.next() {
			let expand = or.to_token_stream(crate_name, field, ty, false);

			or_tokens.extend(quote! {
				let mut __wary_report_inner = #crate_name::error::Report::default();
				{
					let mut __wary_report = &mut __wary_report_inner;
					#expand ;
				}
			});
		}

		for or in or {
			let expand = or.to_token_stream(crate_name, field, ty, false);

			or_tokens.extend(quote! {
				if !__wary_report_inner.is_empty() {
					__wary_report_inner.clear();
					let mut __wary_report = &mut __wary_report_inner;
					#expand ;
				}
			});
		}

		if !self.or.0.is_empty() {
			or_tokens.extend(quote! {
				__wary_report.extend(__wary_report_inner);
			});

			tokens.extend(quote! {
				{
					#or_tokens
				}
			});
		}

		if self.dive.is_present() {
			tokens.extend(quote! {
				#crate_name::Validate::validate_into(#field, ctx, &#error_path, __wary_report);
			});
		}

		if let Some(ref option_path) = option_path {
			let el = self.required.as_ref().map_or_else(
				|| quote!(if false {}),
				|args| {
					quote! {
						if let Err(e) = #crate_name::Rule::validate(
							&#crate_name::options::rule::required::Rule::new() #args,
							&(),
							#field
						) {
							__wary_report.push(#error_path, e);
						};
					}
				},
			);

			return if top {
				quote! {
					if let #option_path ::Some(#field) = #field {
						#tokens
					} else #el
				}
			} else {
				quote! {
					#el
				}
			};
		}

		tokens
	}
}

impl ValidateOptions {
	pub fn into_token_stream(
		self,
		crate_name: &syn::Path,
		ty: &syn::Type,
	) -> proc_macro2::TokenStream {
		ValidateField {
			func: self.func,
			custom: self.custom,
			custom_async: self.custom_async,
			or: self.or,
			and: self.and,
			dive: darling::util::Flag::default(),
			inner: None,
			required: None,
			builtin: Map::default(),
		}
		.to_token_stream(
			crate_name,
			&Field::new_ident(format_ident!("self"), false),
			ty,
			true,
		)
	}
}

impl ValidateFieldWrapper {
	fn into_inner(self) -> ValidateField {
		ValidateField {
			func: self.func,
			or: self.or,
			and: self.and,
			custom: self.custom,
			custom_async: self.custom_async,
			inner: self.inner,
			builtin: self.builtin,
			required: self.required,
			dive: self.dive,
		}
	}
}

impl ValidateFieldWrapper {
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
