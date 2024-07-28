use std::collections::HashMap;

use darling::{ast, FromDeriveInput, FromField, FromMeta};
use quote::{quote, ToTokens};

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

impl<'a> ToTokens for ArgsRef<'a> {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		for (path, value) in &self.0 .0 {
			tokens.extend(quote! {
				.#path(&#value)
			})
		}
	}
}

/// Emits method calls with `Into::into` for every argument.
#[derive(Debug)]
struct ArgsInto<'a>(&'a Args);

impl<'a> ToTokens for ArgsInto<'a> {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		for (path, value) in &self.0 .0 {
			tokens.extend(quote! {
				.#path(#value.into())
			})
		}
	}
}

#[derive(Debug, FromField)]
#[darling(attributes(validate))]
struct ValidateField {
	ident: Option<syn::Ident>,

	email: Option<Args>,
	length: Option<Args>,
	url: Option<Args>,

	range: Option<Args>,
	#[darling(multiple)]
	func: Vec<syn::Expr>,

	// #[darling(multiple)]
	// or: Vec<ValidateField>,
	#[darling(default)]
	custom: HashMap<syn::Path, Option<Args>>,
}

macro_rules! impl_basic_rules {
	(
		$field:ident, $self:ident, $tokens:ident, $crate_name:ident,
		$($name:ident => $struct_:ident),* $(,)?
	) => {
		$(
			if let Some(ref args) = $self.$name {
				let struct_ = syn::Ident::new(stringify!($struct_), proc_macro2::Span::call_site());

				$tokens.extend(quote! {
					#$crate_name::rule::#struct_::new(&$self.#$field)
						#args
						.validate(&())?;
				});
			}
		)*
	};
}

impl ValidateField {
	fn to_token_stream(&self, crate_name: &syn::Path) -> proc_macro2::TokenStream {
		let mut tokens = proc_macro2::TokenStream::new();

		let ident = self.ident.as_ref().unwrap();

		impl_basic_rules! {
			ident, self, tokens, crate_name,
			length => LengthRule,
			email => EmailRule,
			url => UrlRule,
		}

		if let Some(ref args) = self.range {
			// Special handling for strings to avoid allocations when using a range.
			let is_str = args.0.values().any(|v| {
				matches!(
					v,
					Some(syn::Expr::Lit(syn::ExprLit {
						lit: syn::Lit::Str(_),
						..
					}))
				)
			});

			if is_str {
				let args = ArgsInto(args);

				tokens.extend(quote! {
					#crate_name::rule::RangeRule::new(#crate_name::util::DerefStr::deref_str(&self.#ident))
						#args
						.validate(&())?;
				});
			} else {
				let args = ArgsRef(args);

				tokens.extend(quote! {
					#crate_name::rule::RangeRule::new(&self.#ident)
						#args
						.validate(&())?;
				});
			}
		}

		for func in &self.func {
			tokens.extend(quote! {
				{
					let result: Result<(), #crate_name::Error> = (#func)(ctx, &self.#ident);
					result?;
				}
			});
		}

		for (path, args) in &self.custom {
			let args = args.as_ref().map(ArgsRef);

			tokens.extend(quote! {
				<#path as #crate_name::Validate>::new(&self.#ident)
					#args
					.validate(ctx)?;
			});
		}

		tokens
	}
}

fn default_context() -> syn::Type {
	syn::parse_quote! { () }
}

fn default_crate_name() -> syn::Path {
	syn::parse_quote! { ::validator }
}

#[derive(FromDeriveInput)]
#[darling(attributes(validate))]
struct Validate {
	ident: syn::Ident,
	generics: syn::Generics,

	data: ast::Data<(), ValidateField>,

	/// The context type to use when validating.
	#[darling(default = "default_context")]
	context: syn::Type,
	#[darling(default = "default_crate_name", rename = "crate")]
	crate_name: syn::Path,
}

impl ToTokens for Validate {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let Self {
			ref ident,
			ref generics,
			ref data,
			ref crate_name,
			ref context,
		} = self;

		let (imp, ty, wher) = generics.split_for_impl();

		// TODO: support enum
		let fields = data.as_ref().take_struct().unwrap().fields;
		let fields = fields.iter().map(|f| f.to_token_stream(crate_name));

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
}

#[proc_macro_derive(Validate, attributes(validate, serde))]
pub fn validate(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let validate =
		Validate::from_derive_input(&syn::parse_macro_input!(input as syn::DeriveInput)).unwrap();

	quote!(#validate).into()
}
