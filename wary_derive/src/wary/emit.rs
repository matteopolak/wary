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
}

pub struct Emit {
	options: Options,
	validate: Validate,
	transform: Transform,
}

impl Emit {
	pub fn into_token_stream(self) -> TokenStream {
		match (self.validate.data, self.transform.data) {
			(ast::Data::Enum(validate), ast::Data::Enum(transform)) => EmitEnum {
				options: &self.options,
				validate,
				transform,
				validate_top: ValidateOptions {
					func: self.validate.func,
					or: self.validate.or,
					and: self.validate.and,
					custom: self.validate.custom,
				},
				transform_top: TransformOptions {
					func: self.transform.func,
					custom: self.transform.custom,
				},
			}
			.into_token_stream(),
			(ast::Data::Struct(validate), ast::Data::Struct(transform)) => EmitStruct {
				options: &self.options,
				validate,
				transform,
				validate_top: ValidateOptions {
					func: self.validate.func,
					or: self.validate.or,
					and: self.validate.and,
					custom: self.validate.custom,
				},
				transform_top: TransformOptions {
					func: self.transform.func,
					custom: self.transform.custom,
				},
			}
			.into_token_stream(),
			_ => unimplemented!(),
		}
	}
}

impl FromDeriveInput for Emit {
	fn from_derive_input(input: &syn::DeriveInput) -> darling::Result<Self> {
		let options = Options::from_derive_input(input)?;
		let validate = Validate::from_derive_input(input)?;
		let transform = Transform::from_derive_input(input)?;

		Ok(Self {
			options,
			validate,
			transform,
		})
	}
}

struct EmitEnum<'o> {
	options: &'o Options,
	validate: Vec<ValidateVariant>,
	transform: Vec<TransformVariant>,
	validate_top: ValidateOptions,
	transform_top: TransformOptions,
}

impl EmitEnum<'_> {
	fn into_token_stream(self) -> TokenStream {
		let ident = &self.options.ident;

		let validate = self.validate.into_iter().map(|v| {
			let destruct = Fields(&v.fields).destruct();
			let ident = v.ident.clone();

			let fields = Fields(&v.fields)
				.idents()
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
				.idents()
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

		quote! {
			#[allow(warnings)]
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

struct EmitStruct<'o> {
	options: &'o Options,
	validate: ast::Fields<ValidateFieldWrapper>,
	transform: ast::Fields<TransformFieldWrapper>,
	validate_top: ValidateOptions,
	transform_top: TransformOptions,
}

impl EmitStruct<'_> {
	fn into_token_stream(self) -> TokenStream {
		let destruct = Fields(&self.validate).destruct();
		let idents = Fields(&self.validate).idents();
		let ident = &self.options.ident;

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

		quote! {
			#[allow(warnings)]
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
