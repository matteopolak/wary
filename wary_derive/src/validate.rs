use darling::{FromDeriveInput, FromField, FromMeta, FromVariant, ast};
use quote::{ToTokens, format_ident, quote};

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
	pub custom: Map<syn::Path, Option<Args>>,
}

pub struct ValidateOptions {
	pub func: Vec<syn::Expr>,
	pub or: Tuple<ValidateField>,
	pub custom: Map<syn::Path, Option<Args>>,
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
	custom: Map<syn::Path, Option<Args>>,

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
		let option_path = crate::attr::extract_option_path(ty);

		let field_path = field.path();

		if option_path.is_none() {
			if let Some(args) = &self.required {
				tokens.extend(quote! {
					if let Err(e) = #crate_name::Rule::validate(
						&#crate_name::options::rule::required::Rule::new() #args,
						&(),
						#field
					) {
						__wary_report.push(__wary_parent.append(#field_path), e);
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
										static PAT: ::std::sync::LazyLock<#crate_name::options::rule::#path::Regex> =
											::std::sync::LazyLock::new(|| {
												#crate_name::options::rule::#path::Regex::new(#s).unwrap()
											});

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
					__wary_report.push(__wary_parent.append(#field_path), e);
				};
			});
		}

		if let Some(inner) = &mut self.inner {
			let inner = inner.to_token_stream(crate_name, field, ty, false);

			tokens.extend(quote! {
				if let Err(e) = (|| {
					for #field in #crate_name::AsSlice::as_slice(#field) {
						#inner
					}

					Ok::<(), #crate_name::Error>(())
				})() {
					__wary_report.push(__wary_parent.append(#field_path), e);
				};
			});
		}

		for func in &self.func {
			tokens.extend(quote! {
				{
					let result: Result<(), #crate_name::Error> = (#func)(ctx, #field);
					if let Err(e) = result {
						__wary_report.push(__wary_parent.append(#field_path), e);
					};
				}
			});
		}

		for (path, args) in self.custom.iter() {
			tokens.extend(quote! {
				if let Err(e) = #crate_name::Rule::validate(
					&#path::new() #args,
					ctx,
					#field
				) {
					__wary_report.push(__wary_parent.append(#field_path), e);
				};
			});
		}

		for and in &mut self.and.0 {
			let expand = and.to_token_stream(crate_name, field, ty, false);

			tokens.extend(quote! {
				#expand ;
			});
		}

		let mut or = self.or.0.iter_mut();

		if let Some(or) = or.next() {
			let expand = or.to_token_stream(crate_name, field, ty, false);

			tokens.extend(quote! {
				let __wary_last: Result<(), #crate_name::Error> = (|| {
					#expand ;
					Ok(())
				})();
			});
		}

		for or in or {
			let expand = or.to_token_stream(crate_name, field, ty, false);

			tokens.extend(quote! {
				let __wary_last: Result<(), #crate_name::Error> = __wary_last.or_else(|_| {
					#expand ;
					Ok(())
				});
			});
		}

		if !self.or.0.is_empty() {
			tokens.extend(quote! {
				if let Err(e) = __wary_last {
					__wary_report.push(__wary_parent.append(#field_path), e);
				};
			});
		}

		if self.dive.is_present() {
			tokens.extend(quote! {
				#crate_name::Validate::validate_into(#field, ctx, &__wary_parent.append(#field_path), __wary_report);
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
							__wary_report.push(__wary_parent.append(#field_path), e);
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
			or: self.or,
			and: Tuple::default(),
			dive: darling::util::Flag::default(),
			inner: None,
			required: None,
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

impl ValidateFieldWrapper {
	fn into_inner(self) -> ValidateField {
		ValidateField {
			func: self.func,
			or: self.or,
			and: Tuple::default(),
			custom: self.custom,
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
