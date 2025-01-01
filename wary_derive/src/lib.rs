mod attr;

use std::collections::HashMap;

use darling::{ast, FromDeriveInput, FromField, FromMeta, FromVariant};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::punctuated::Punctuated;

#[derive(Debug, Default)]
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

enum Arg {
	Name(syn::Path),
	NameValue(syn::MetaNameValue),
	Range(syn::ExprRange),
}

impl syn::parse::Parse for Arg {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		if input.peek(syn::Ident) {
			let within = input.fork();
			let path = within.parse::<syn::Path>()?;

			return Ok(if within.peek(syn::token::Eq) {
				input.parse::<syn::Path>()?;

				let eq = input.parse::<syn::token::Eq>()?;

				Self::NameValue(syn::MetaNameValue {
					path,
					eq_token: eq,
					value: input.parse()?,
				})
			} else if within.peek(syn::token::Comma) || within.peek(syn::token::Paren) {
				input.parse::<syn::Path>()?;

				Self::Name(path)
			} else {
				Self::Range(input.parse::<syn::ExprRange>()?)
			});
		}

		if let Ok(range) = input.parse::<syn::ExprRange>() {
			// otherwise, check if it's range syntax and expand it into a few different
			// arguments l..h -> min = l, exclusive_max = h
			// l..=h -> min = l, max = h
			// ..h -> exclusive_max = h
			// ..=h -> max = h
			//
			// unfortuntely cannot support exclusive min with this syntax, but it could be
			// provided manually with a simple ..=h and exclusive_min=...

			return Ok(Self::Range(range));
		}

		Err(input.error("expected `=` or range syntax"))
	}
}

impl FromMeta for Args {
	fn from_meta(item: &syn::Meta) -> Result<Self, darling::Error> {
		let mut map = HashMap::new();

		let metas = item
			.require_list()?
			.parse_args_with(Punctuated::<Arg, syn::Token![,]>::parse_terminated)?;

		for meta in metas {
			match meta {
				Arg::NameValue(meta) => {
					map.insert(meta.path, Some(meta.value));
				}
				Arg::Name(path) => {
					map.insert(path, None);
				}
				Arg::Range(range) => {
					if let Some(start) = range.start {
						map.insert(syn::parse_quote! { min }, Some(*start));
					}

					match range.limits {
						syn::RangeLimits::HalfOpen(..) => {
							if let Some(end) = range.end {
								map.insert(syn::parse_quote! { exclusive_max }, Some(*end));
							}
						}
						syn::RangeLimits::Closed(..) => {
							if let Some(end) = range.end {
								map.insert(syn::parse_quote! { max }, Some(*end));
							}
						}
					}
				}
			}
		}

		Ok(Self(map))
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

	#[darling(default)]
	inner: Option<Box<ValidateField>>,

	#[darling(flatten)]
	builtin: HashMap<syn::Path, Option<Args>>,
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

	#[darling(default)]
	inner: Option<Box<ValidateField>>,

	#[darling(flatten)]
	builtin: HashMap<syn::Path, Option<Args>>,
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

impl ValidateField {
	fn to_token_stream(
		&mut self,
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

		for (path, args) in &mut self.builtin {
			let args_ref = args.as_ref().map(ArgsRef);
			let args: &dyn ToTokens = if path.is_ident("range") {
				&args_ref
			} else if path.is_ident("matches") {
				let key = syn::parse_quote! { pat };

				if let Some(args) = args {
					if let Some(Some(expr)) = args.0.get(&key) {
						if let Some(s) = attr::extract_str(expr) {
							args.0.insert(
								key,
								Some(syn::parse_quote! {
									{
										static PAT: ::std::sync::LazyLock<#crate_name::rule::matches::Regex> =
											::std::sync::LazyLock::new(|| {
												#crate_name::rule::matches::Regex::new(#s).unwrap()
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
					#crate_name::rule::#path::Rule::new(field)
					#args
					.validate(&())?;
			});
		}

		if let Some(inner) = &mut self.inner {
			let inner = inner.to_token_stream(crate_name, &quote!(field), ty, false);

			tokens.extend(quote! {
				#crate_name::rule::inner::Rule::new(field, |field| {
					#inner

					Ok::<(), #crate_name::Error>(())
				})
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
			inner: self.inner,
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
