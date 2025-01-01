use darling::{ast, FromDeriveInput, FromField, FromMeta, FromVariant};
use quote::{quote, ToTokens};

use crate::{
	attr,
	util::{Args, ArgsRef, Map, Tuple},
};

#[derive(FromDeriveInput)]
#[darling(attributes(validate))]
pub struct Validate {
	pub data: ast::Data<ValidateVariant, ValidateFieldWrapper>,
}

#[derive(Debug, FromVariant)]
pub struct ValidateVariant {
	pub ident: syn::Ident,
	pub fields: ast::Fields<ValidateFieldWrapper>,
}

#[derive(Debug, FromMeta)]
struct ValidateField {
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

	#[darling(flatten)]
	builtin: Map<syn::Path, Option<Args>>,
}

impl ValidateField {
	#[allow(clippy::wrong_self_convention)]
	fn to_token_stream(
		&mut self,
		crate_name: &syn::Path,
		field: &syn::Ident,
		ty: &syn::Type,
		top: bool,
	) -> proc_macro2::TokenStream {
		let mut tokens = proc_macro2::TokenStream::new();
		let option_path = crate::attr::extract_option_path(ty);

		for (path, args) in self.builtin.iter_mut() {
			let args_ref = args.as_ref().map(ArgsRef);
			let args: &dyn ToTokens = if path.is_ident("range") {
				&args_ref
			} else if path.is_ident("matches") {
				let key = syn::parse_quote! { pat };

				if let Some(args) = args {
					if let Some(Some(expr)) = args.get(&key) {
						if let Some(s) = attr::extract_str(expr) {
							args.insert(
								key,
								Some(syn::parse_quote! {
									{
										static PAT: ::std::sync::LazyLock<#crate_name::options::rule::matches::Regex> =
											::std::sync::LazyLock::new(|| {
												#crate_name::options::rule::matches::Regex::new(#s).unwrap()
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
				#crate_name::Rule::validate(
					&#crate_name::options::rule::#path::Rule::new() #args,
					&(),
					#field
				)?;
			});
		}

		if let Some(inner) = &mut self.inner {
			let inner = inner.to_token_stream(crate_name, field, ty, false);

			tokens.extend(quote! {
				#crate_name::Rule::validate(
					&#crate_name::options::rule::inner::Rule::new(|field| {
						#inner

						Ok::<(), #crate_name::Error>(())
					}),
					&(),
					#field
				)?;
			});
		}

		for func in &self.func {
			tokens.extend(quote! {
				{
					let result: Result<(), #crate_name::Error> = (#func)(ctx, #field);
					result?;
				}
			});
		}

		for (path, args) in self.custom.iter() {
			tokens.extend(quote! {
				#crate_name::Rule::validate(
					&#path::new() #args,
					ctx,
					#field
				)?;
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
				__wary_last?;
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

impl ValidateFieldWrapper {
	fn into_inner(self) -> ValidateField {
		ValidateField {
			func: self.func,
			or: self.or,
			and: Tuple::default(),
			custom: self.custom,
			inner: self.inner,
			builtin: self.builtin,
		}
	}
}

impl ValidateFieldWrapper {
	pub fn into_token_stream(
		self,
		crate_name: &syn::Path,
		field: &syn::Ident,
	) -> proc_macro2::TokenStream {
		let ty = self.ty.clone();
		self
			.into_inner()
			.to_token_stream(crate_name, field, &ty, true)
	}
}
