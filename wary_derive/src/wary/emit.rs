use darling::{ast, FromDeriveInput};
use proc_macro2::TokenStream;
use quote::quote;

use super::{
	transform::{Transform, TransformFieldWrapper, TransformOptions, TransformVariant},
	validate::{Validate, ValidateFieldWrapper, ValidateOptions, ValidateVariant},
};
use crate::util::Fields;

fn default_context() -> Type {
	Type(syn::parse_quote! { () })
}

struct Type(syn::Type);

impl darling::FromMeta for Type {
	fn from_string(value: &str) -> darling::Result<Self> {
		syn::parse_str(value)
			.map(Self)
			.map_err(|_| darling::Error::unknown_value(value))
	}

	fn from_value(value: &syn::Lit) -> darling::Result<Self> {
		if let syn::Lit::Str(ref v) = *value {
			v.parse()
				.map(Self)
				.map_err(|_| darling::Error::unknown_value(&v.value()).with_span(v))
		} else {
			Err(darling::Error::unexpected_lit_type(value))
		}
	}

	fn from_meta(item: &syn::Meta) -> darling::Result<Self> {
		let syn::Expr::Path(p) = &item.require_name_value()?.value else {
			return Err(darling::Error::unknown_value("expected a type"));
		};

		Ok(Self(syn::Type::Path(syn::TypePath {
			qself: p.qself.clone(),
			path: p.path.clone(),
		})))
	}
}

impl darling::ToTokens for Type {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		self.0.to_tokens(tokens);
	}
}

#[derive(FromDeriveInput)]
#[darling(attributes(wary))]
pub struct Options {
	ident: syn::Ident,
	generics: syn::Generics,

	/// The context type to use when validating.
	#[darling(default = "default_context")]
	context: Type,
	#[darling(default = "crate::default_crate_name", rename = "crate")]
	crate_name: syn::Path,
	transparent: darling::util::Flag,
}

#[cfg(feature = "serde")]
pub mod serde {
	pub type Container<'a> = serde_derive_internals::ast::Container<'a>;
	pub type Data<'a> = serde_derive_internals::ast::Data<'a>;
	pub type Variant<'a> = serde_derive_internals::ast::Variant<'a>;
	pub type Field<'a> = serde_derive_internals::ast::Field<'a>;
}

#[cfg(not(feature = "serde"))]
pub mod serde {
	use core::marker::PhantomData;

	pub struct Container<'a> {
		pub data: Data<'a>,
	}

	pub type Variant<'a> = PhantomData<&'a ()>;
	pub type Field<'a> = PhantomData<&'a ()>;

	pub enum Data<'a> {
		Struct((), Vec<PhantomData<&'a ()>>),
		Enum(Vec<PhantomData<&'a ()>>),
	}
}

pub struct Emit<'a> {
	options: Options,
	validate: Validate,
	transform: Transform,
	serde: serde::Container<'a>,
}

impl<'a> Emit<'a> {
	pub fn into_token_stream(self) -> TokenStream {
		match (self.validate.data, self.transform.data, self.serde.data) {
			(ast::Data::Enum(validate), ast::Data::Enum(transform), serde::Data::Enum(serde)) => {
				EmitEnum {
					options: &self.options,
					serde,
					validate,
					transform,
					validate_top: ValidateOptions {
						func: self.validate.func,
						or: self.validate.or,
						and: self.validate.and,
						custom: self.validate.custom,
						custom_async: self.validate.custom_async,
					},
					transform_top: TransformOptions {
						func: self.transform.func,
						custom: self.transform.custom,
						custom_async: self.transform.custom_async,
					},
				}
				.into_token_stream()
			}
			(
				ast::Data::Struct(validate),
				ast::Data::Struct(transform),
				serde::Data::Struct(_, serde),
			) => EmitStruct {
				options: &self.options,
				serde,
				validate,
				transform,
				validate_top: ValidateOptions {
					func: self.validate.func,
					or: self.validate.or,
					and: self.validate.and,
					custom: self.validate.custom,
					custom_async: self.validate.custom_async,
				},
				transform_top: TransformOptions {
					func: self.transform.func,
					custom: self.transform.custom,
					custom_async: self.transform.custom_async,
				},
			}
			.into_token_stream(),
			_ => unimplemented!(),
		}
	}

	pub fn from_derive_input(input: &'a mut syn::DeriveInput) -> darling::Result<Self> {
		let options = Options::from_derive_input(input)?;
		let validate = Validate::from_derive_input(input)?;
		let transform = Transform::from_derive_input(input)?;

		#[cfg(feature = "serde")]
		let cont = {
			serde_derive_internals::replace_receiver(input);
			let ctxt = serde_derive_internals::Ctxt::new();
			let Some(cont) = serde_derive_internals::ast::Container::from_ast(
				&ctxt,
				input,
				serde_derive_internals::Derive::Deserialize,
			) else {
				return Err(ctxt.check().unwrap_err().into());
			};
			ctxt.check()?;
			cont
		};
		#[cfg(not(feature = "serde"))]
		let cont = serde::Container {
			data: if validate.data.is_enum() {
				serde::Data::Enum(Vec::new())
			} else {
				serde::Data::Struct((), Vec::new())
			},
		};

		Ok(Self {
			options,
			validate,
			transform,
			serde: cont,
		})
	}
}

struct EmitEnum<'o> {
	options: &'o Options,
	serde: Vec<serde::Variant<'o>>,
	validate: Vec<ValidateVariant>,
	transform: Vec<TransformVariant>,
	validate_top: ValidateOptions,
	transform_top: TransformOptions,
}

impl EmitEnum<'_> {
	fn into_token_stream(self) -> TokenStream {
		if self.options.transparent.is_present() {
			return darling::Error::custom("transparent enums are not supported").write_errors();
		}

		let ident = &self.options.ident;

		let is_validate_async = self
			.validate
			.iter()
			.any(|v| !v.fields.iter().any(|f| f.custom_async.is_empty()))
			|| !self.validate_top.custom_async.is_empty();

		let is_transform_async = self
			.transform
			.iter()
			.any(|m| !m.fields.iter().any(|f| f.custom_async.is_empty()))
			|| !self.transform_top.custom_async.is_empty();

		#[allow(unused)]
		let validate = self.validate.into_iter().zip(self.serde).map(|(v, serde)| {
			let destruct = Fields(&v.fields).destruct();
			let ident = v.ident.clone();

			#[cfg(feature = "serde")]
			let serde_fields = Some(serde.fields);
			#[cfg(not(feature = "serde"))]
			let serde_fields = None;

			let fields = Fields(&v.fields)
				.idents(serde_fields, false)
				.into_iter()
				.zip(v.fields)
				.map(|(field, f)| f.into_token_stream(&self.options.crate_name, &field));

			quote! {
				Self::#ident { #destruct } => {
					#(
						#fields
					)*
				}
			}
		});

		let validate_top = self
			.validate_top
			.into_token_stream(&self.options.crate_name, &syn::parse_quote!(#ident));

		let transform = self.transform.into_iter().map(|m| {
			let destruct = Fields(&m.fields).destruct();
			let ident = m.ident.clone();

			let fields = Fields(&m.fields)
				.idents(None, false)
				.into_iter()
				.zip(m.fields)
				.map(|(field, m)| m.into_token_stream(&self.options.crate_name, &field));

			quote! {
				Self::#ident { #destruct } => {
					#(
						#fields
					)*
				}
			}
		});

		let transform_top = self
			.transform_top
			.into_token_stream(&self.options.crate_name, &syn::parse_quote!(#ident));

		let (imp, ty, wher) = self.options.generics.split_for_impl();
		let crate_name = &self.options.crate_name;
		let context = &self.options.context;
		let ident = &self.options.ident;

		if is_validate_async || is_transform_async {
			quote! {
				#[allow(warnings)]
				#[automatically_derived]
				impl #imp #crate_name::AsyncValidate for #ident #ty #wher {
					type Context = #context;

					async fn validate_into_async(&self, ctx: &Self::Context, __wary_parent: &#crate_name::error::Path, __wary_report: &mut #crate_name::error::Report) {
						match self {
							#(
								#validate
							)*
						};

						#validate_top
					}
				}

				#[allow(warnings)]
				#[automatically_derived]
				impl #imp #crate_name::AsyncTransform for #ident #ty #wher {
					type Context = #context;

					async fn transform_async(&mut self, ctx: &Self::Context) {
						match self {
							#(
								#transform
							)*
						};

						#transform_top
					}
				}
			}
		} else {
			quote! {
				#[allow(warnings)]
				#[automatically_derived]
				impl #imp #crate_name::Validate for #ident #ty #wher {
					type Context = #context;

					fn validate_into(&self, ctx: &Self::Context, __wary_parent: &#crate_name::error::Path, __wary_report: &mut #crate_name::error::Report) {
						match self {
							#(
								#validate
							)*
						};

						#validate_top
					}
				}

				#[allow(warnings)]
				#[automatically_derived]
				impl #imp #crate_name::Transform for #ident #ty #wher {
					type Context = #context;

					fn transform(&mut self, ctx: &Self::Context) {
						match self {
							#(
								#transform
							)*
						};

						#transform_top
					}
				}
			}
		}
	}
}

struct EmitStruct<'o> {
	options: &'o Options,
	serde: Vec<serde::Field<'o>>,
	validate: ast::Fields<ValidateFieldWrapper>,
	transform: ast::Fields<TransformFieldWrapper>,
	validate_top: ValidateOptions,
	transform_top: TransformOptions,
}

impl EmitStruct<'_> {
	fn into_token_stream(self) -> TokenStream {
		if self.options.transparent.is_present() && self.validate.fields.len() != 1 {
			return darling::Error::custom("transparent structs must have exactly one field")
				.write_errors();
		}

		let destruct = Fields(&self.validate).destruct();
		let idents =
			Fields(&self.validate).idents(Some(self.serde), self.options.transparent.is_present());
		let ident = &self.options.ident;

		let is_validate_async = self.validate.iter().any(|v| !v.custom_async.is_empty())
			|| !self.validate_top.custom_async.is_empty();

		let is_transform_async = self.transform.iter().any(|m| !m.custom_async.is_empty())
			|| !self.transform_top.custom_async.is_empty();

		let validate = self
			.validate
			.into_iter()
			.zip(&idents)
			.map(|(v, i)| v.into_token_stream(&self.options.crate_name, i));

		let validate_top = self
			.validate_top
			.into_token_stream(&self.options.crate_name, &syn::parse_quote!(#ident));

		let transform = self
			.transform
			.into_iter()
			.zip(&idents)
			.map(|(m, i)| m.into_token_stream(&self.options.crate_name, i));

		let transform_top = self
			.transform_top
			.into_token_stream(&self.options.crate_name, &syn::parse_quote!(#ident));

		let (imp, ty, wher) = self.options.generics.split_for_impl();
		let crate_name = &self.options.crate_name;
		let context = &self.options.context;
		let ident = &self.options.ident;

		if is_validate_async || is_transform_async {
			quote! {
				#[allow(warnings)]
				#[automatically_derived]
				impl #imp #crate_name::AsyncValidate for #ident #ty #wher {
					type Context = #context;

					async fn validate_into_async(&self, ctx: &Self::Context, __wary_parent: &#crate_name::error::Path, __wary_report: &mut #crate_name::error::Report) {
						let Self { #destruct } = self;

						#(
							#validate
						)*

						#validate_top
					}
				}

				#[allow(warnings)]
				#[automatically_derived]
				impl #imp #crate_name::AsyncTransform for #ident #ty #wher {
					type Context = #context;

					async fn transform_async(&mut self, ctx: &Self::Context) {
						let Self { #destruct } = self;

						#(
							#transform
						)*

						#transform_top
					}
				}
			}
		} else {
			quote! {
				#[allow(warnings)]
				#[automatically_derived]
				impl #imp #crate_name::Validate for #ident #ty #wher {
					type Context = #context;

					fn validate_into(&self, ctx: &Self::Context, __wary_parent: &#crate_name::error::Path, __wary_report: &mut #crate_name::error::Report) {
						let Self { #destruct } = self;

						#(
							#validate
						)*

						#validate_top
					}
				}

				#[allow(warnings)]
				#[automatically_derived]
				impl #imp #crate_name::Transform for #ident #ty #wher {
					type Context = #context;

					fn transform(&mut self, ctx: &Self::Context) {
						let Self { #destruct } = self;

						#(
							#transform
						)*

						#transform_top
					}
				}
			}
		}
	}
}
