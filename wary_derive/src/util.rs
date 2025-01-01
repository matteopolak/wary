use std::{collections::HashSet, ops};

use darling::{ast, FromMeta};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::punctuated::Punctuated;

use crate::{modify::ModifyFieldWrapper, validate::ValidateFieldWrapper};

pub type Map<K, V> = VecMap<K, V>;

/// Avoids depending on `indexmap` since performance does
/// not matter much here.
#[derive(Debug)]
pub struct VecMap<K, V> {
	inner: Vec<(K, V)>,
}

impl<K, V> ops::Deref for VecMap<K, V> {
	type Target = [(K, V)];

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl<K, V> ops::DerefMut for VecMap<K, V> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}

impl<K, V> Default for VecMap<K, V> {
	fn default() -> Self {
		Self::new()
	}
}

impl<K, V> VecMap<K, V> {
	pub fn new() -> Self {
		Self { inner: Vec::new() }
	}

	pub fn with_capacity(capacity: usize) -> Self {
		Self {
			inner: Vec::with_capacity(capacity),
		}
	}

	pub fn insert(&mut self, key: K, value: V)
	where
		K: PartialEq,
	{
		if let Some(v) = self.get_mut(&key) {
			*v = value;
		} else {
			self.inner.push((key, value));
		}
	}

	pub fn get(&self, key: &K) -> Option<&V>
	where
		K: PartialEq,
	{
		self
			.inner
			.iter()
			.find_map(|(k, v)| if k == key { Some(v) } else { None })
	}

	pub fn get_mut(&mut self, key: &K) -> Option<&mut V>
	where
		K: PartialEq,
	{
		self
			.inner
			.iter_mut()
			.find_map(|(k, v)| if k == key { Some(v) } else { None })
	}
}

impl<V: FromMeta> FromMeta for VecMap<syn::Path, V> {
	fn from_list(nested: &[ast::NestedMeta]) -> darling::Result<Self> {
		let pairs = nested.iter().map(
			|item| -> darling::Result<(&syn::Path, darling::Result<V>)> {
				match *item {
					ast::NestedMeta::Meta(ref inner) => {
						let path = inner.path();

						Ok((
							path,
							FromMeta::from_meta(inner).map_err(|e| e.at_path(path)),
						))
					}
					ast::NestedMeta::Lit(_) => Err(darling::Error::unsupported_format("expression")),
				}
			},
		);

		let mut errors = darling::Error::accumulator();
		let mut seen_keys = HashSet::with_capacity(nested.len());
		let mut map = Self::with_capacity(nested.len());

		for item in pairs {
			if let Some((path, value)) = errors.handle(item) {
				let already_seen = seen_keys.contains(path);

				if already_seen {
					errors.push(
						darling::Error::duplicate_field(&darling::util::path_to_string(path)).with_span(path),
					);
				}

				match value {
					Ok(_) if already_seen => {}
					Ok(val) => {
						map.insert(path.clone(), val);
					}
					Err(e) => {
						errors.push(e);
					}
				}

				seen_keys.insert(path.clone());
			}
		}

		errors.finish_with(map)
	}
}

#[derive(Debug, Default)]
pub struct Args(Map<syn::Path, Option<syn::Expr>>);

impl Args {
	pub fn insert(&mut self, key: syn::Path, value: Option<syn::Expr>) {
		self.0.insert(key, value);
	}

	pub fn get(&self, key: &syn::Path) -> Option<&Option<syn::Expr>> {
		self.0.get(key)
	}
}

impl<'d> IntoIterator for &'d Args {
	type IntoIter = std::slice::Iter<'d, (syn::Path, Option<syn::Expr>)>;
	type Item = &'d (syn::Path, Option<syn::Expr>);

	fn into_iter(self) -> Self::IntoIter {
		self.0.inner.iter()
	}
}

impl ToTokens for Args {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		for (path, value) in &self.0.inner {
			tokens.extend(quote! {
				.#path(#value)
			})
		}
	}
}

#[derive(Debug)]
pub struct Tuple<T>(pub Vec<T>);

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
		let mut map = Map::new();

		if let syn::Meta::Path(..) = item {
			return Ok(Self(map));
		}

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

/// Emits method calls with references instead of values.
#[derive(Debug)]
pub struct ArgsRef<'a>(pub &'a Args);

impl ToTokens for ArgsRef<'_> {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		for (path, value) in self.0 {
			tokens.extend(quote! {
				.#path(&#value)
			})
		}
	}
}

pub struct Fields<'f, F>(pub &'f ast::Fields<F>);

pub trait Identify {
	fn ident(&self) -> Option<&syn::Ident>;
}

impl Identify for ValidateFieldWrapper {
	fn ident(&self) -> Option<&syn::Ident> {
		self.ident.as_ref()
	}
}

impl Identify for ModifyFieldWrapper {
	fn ident(&self) -> Option<&syn::Ident> {
		self.ident.as_ref()
	}
}

impl<F> Fields<'_, F>
where
	F: Identify,
{
	/// Emits the destructuring of the variant to be used in a match arm within
	/// e.g. `Self::Variant { #here }`.
	pub fn destruct(&self) -> TokenStream {
		let idents = self
			.0
			.iter()
			.enumerate()
			.map(|(i, f)| {
				f.ident()
					.map_or_else(|| (true, format_ident!("_{i}")), |f| (false, f.clone()))
			})
			.collect::<Vec<_>>();

		idents
			.iter()
			.enumerate()
			.map(|(i, (is_tuple, ident))| {
				if *is_tuple {
					quote! { #i: #ident }
				} else {
					ident.to_token_stream()
				}
			})
			.collect()
	}

	pub fn idents(&self) -> Vec<syn::Ident> {
		self
			.0
			.iter()
			.enumerate()
			.map(|(i, f)| {
				f.ident()
					.map_or_else(|| format_ident!("_{i}"), |f| f.clone())
			})
			.collect::<Vec<_>>()
	}
}
