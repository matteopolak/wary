mod attr;

use std::collections::HashMap;

use darling::{ast, FromDeriveInput, FromField, FromMeta, FromVariant};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::punctuated::Punctuated;

#[derive(Debug)]
struct Args(HashMap<syn::Path, Option<syn::Expr>>);

impl ToTokens for Args {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		for (path, value) in &self.0 {
			tokens.extend(quote! {
				.#path(#value)
			})
		}
	}
}

impl FromMeta for Args {
	fn from_meta(item: &syn::Meta) -> Result<Self, darling::Error> {
		let mut map = HashMap::new();

		// for each item, if it's a key-value pair, then parse as normal.
		// if it's just a key, use the default V and insert it.
		if let syn::Meta::List(ref list) = item {
			list
				.parse_nested_meta(|meta| {
					let value = if meta.input.peek(syn::token::Eq) {
						meta.input.parse::<syn::token::Eq>()?;
						Some(meta.input.parse()?)
					} else {
						None
					};

					map.insert(meta.path, value);

					Ok(())
				})
				.unwrap();
		}

		Ok(Self(map))
	}
}

/// Emits method calls with references instead of values.
#[derive(Debug)]
struct ArgsRef<'a>(&'a Args);

impl ToTokens for ArgsRef<'_> {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		for (path, value) in &self.0 .0 {
			tokens.extend(quote! {
				.#path(&#value)
			})
		}
	}
}

#[derive(Debug)]
struct Tuple<T>(Vec<T>);

impl<T> Default for Tuple<T> {
	fn default() -> Self {
		Self(Vec::default())
	}
}

impl<T> FromMeta for Tuple<T>
where
	T: FromMeta,
{
	fn from_meta(item: &syn::Meta) -> Result<Self, darling::Error> {
		let mut vec = Vec::new();
		let metas = item
			.require_list()?
			.parse_args_with(Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated)?;

		for meta in metas {
			vec.push(T::from_list(&[ast::NestedMeta::Meta(meta)])?);
		}

		Ok(Self(vec))
	}
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
	custom: HashMap<syn::Path, Option<Args>>,

	#[darling(flatten)]
	builtin: HashMap<syn::Path, Args>,
}

#[derive(Debug, FromField)]
#[darling(attributes(validate))]
struct ValidateFieldWrapper {
	ident: Option<syn::Ident>,
	ty: syn::Type,

	#[darling(multiple)]
	func: Vec<syn::Expr>,

	#[darling(default)]
	or: Tuple<ValidateField>,

	#[darling(default)]
	custom: HashMap<syn::Path, Option<Args>>,

	#[darling(flatten)]
	builtin: HashMap<syn::Path, Args>,
}

impl ValidateField {
	fn to_token_stream(
		&self,
		crate_name: &syn::Path,
		field: &TokenStream,
		ty: &syn::Type,
		top: bool,
	) -> proc_macro2::TokenStream {
		let mut tokens = proc_macro2::TokenStream::new();
		let option_path = crate::attr::extract_option_path(ty);

		if option_path.is_none() {
			tokens.extend(quote! {
				let field = #field;
			});
		}

		for (path, args) in &self.builtin {
			tokens.extend(quote! {
					#crate_name::rule::#path::Rule::new(field)
					#args
					.validate(&())?;
			});
		}

		for func in &self.func {
			tokens.extend(quote! {
				{
					let result: Result<(), #crate_name::Error> = (#func)(ctx, field);
					result?;
				}
			});
		}

		for (path, args) in &self.custom {
			tokens.extend(quote! {
				#crate_name::Validate::validate(
					&#path::new(field) #args,
					ctx
				)?;
			});
		}

		for and in &self.and.0 {
			let expand = and.to_token_stream(crate_name, field, ty, false);

			tokens.extend(quote! {
				#expand ;
			});
		}

		let mut or = self.or.0.iter();

		if let Some(or) = or.next() {
			let expand = or.to_token_stream(crate_name, field, ty, false);

			tokens.extend(quote! {
				let last: Result<(), #crate_name::Error> = (|| {
					#expand ;
					Ok(())
				})();
			});
		}

		for or in or {
			let expand = or.to_token_stream(crate_name, field, ty, false);

			tokens.extend(quote! {
				let last: Result<(), #crate_name::Error> = last.or_else(|_| {
					#expand ;
					Ok(())
				});
			});
		}

		if !self.or.0.is_empty() {
			tokens.extend(quote! {
				last?;
			});
		}

		if top {
			if let Some(ref option_path) = option_path {
				return quote! {
					if let #option_path ::Some(field) = #field {
						#tokens
					}
				};
			}
		}

		tokens
	}
}

impl ValidateFieldWrapper {
	pub fn into_inner(self) -> ValidateField {
		ValidateField {
			func: self.func,
			or: self.or,
			and: Tuple::default(),
			custom: self.custom,
			builtin: self.builtin,
		}
	}
}

impl ValidateFieldWrapper {
	fn into_token_stream(
		self,
		crate_name: &syn::Path,
		field: &TokenStream,
	) -> proc_macro2::TokenStream {
		let ty = self.ty.clone();
		self
			.into_inner()
			.to_token_stream(crate_name, field, &ty, true)
	}
}

#[derive(Debug, FromVariant)]
struct ValidateVariant {
	ident: syn::Ident,
	fields: ast::Fields<ValidateFieldWrapper>,
}

fn default_context() -> syn::Type {
	syn::parse_quote! { () }
}

fn default_crate_name() -> syn::Path {
	syn::parse_quote! { ::wary }
}

#[derive(FromDeriveInput)]
#[darling(attributes(validate))]
struct Validate {
	ident: syn::Ident,
	generics: syn::Generics,

	data: ast::Data<ValidateVariant, ValidateFieldWrapper>,

	/// The context type to use when validating.
	#[darling(default = "default_context")]
	context: syn::Type,
	#[darling(default = "default_crate_name", rename = "crate")]
	crate_name: syn::Path,
}

impl Validate {
	fn into_tokens(self, tokens: &mut proc_macro2::TokenStream) {
		let (imp, ty, wher) = self.generics.split_for_impl();

		let crate_name = &self.crate_name;
		let context = &self.context;
		let ident = &self.ident;

		match self.data {
			ast::Data::Enum(variants) => {
				let variants_len = variants.len();
				let variants = variants.into_iter().map(|v| {
					let ident = &v.ident;

					let idents = v
						.fields
						.iter()
						.enumerate()
						.map(|(i, f)| {
							f.ident.as_ref().map_or_else(
								|| (true, format_ident!("_{i}").into_token_stream()),
								|f| (false, ToTokens::to_token_stream(f)),
							)
						})
						.collect::<Vec<_>>();

					let fields = v
						.fields
						.into_iter()
						.zip(idents.iter())
						.map(|(f, (_, field))| f.into_token_stream(crate_name, field));

					let destruct = idents.iter().enumerate().map(|(i, (is_tuple, ident))| {
						if *is_tuple {
							quote! { #i: #ident }
						} else {
							ident.clone()
						}
					});

					quote! {
						Self::#ident { #(#destruct),* } => {
							#(
								#fields
							)*
						}
					}
				});

				let variants = if variants_len == 0 {
					None
				} else {
					Some(quote! {
						match self {
							#(
								#variants
							)*
						};
					})
				};

				tokens.extend(quote! {
					impl #imp #crate_name::Validate for #ident #ty #wher {
						type Context = #context;

						fn validate(&self, ctx: &Self::Context) -> Result<(), #crate_name::Error> {
							#variants

							Ok(())
						}
					}
				});
			}
			ast::Data::Struct(struct_) => {
				let fields = struct_.fields.into_iter().enumerate().map(|(i, f)| {
					let ident = f
						.ident
						.as_ref()
						.map_or_else(|| quote!(&self.#i), |f| quote!(&self.#f));

					f.into_token_stream(crate_name, &ident)
				});

				tokens.extend(quote! {
					impl #imp #crate_name::Validate for #ident #ty #wher {
						type Context = #context;

						fn validate(&self, ctx: &Self::Context) -> Result<(), #crate_name::Error> {
							#(
								#fields
							)*

							Ok(())
						}
					}
				});
			}
		};
	}
}

#[proc_macro_derive(Validate, attributes(validate, serde))]
pub fn validate(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = syn::parse_macro_input!(input as syn::DeriveInput);

	match Validate::from_derive_input(&input) {
		Ok(validate) => {
			let mut stream = TokenStream::new();

			validate.into_tokens(&mut stream);
			stream.into()
		}
		Err(e) => e.write_errors().into(),
	}
}
