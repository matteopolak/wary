use darling::{FromDeriveInput, ast};
use proc_macro2::TokenStream;
use quote::quote;

use crate::{
	modify::{Modify, ModifyFieldWrapper, ModifyOptions, ModifyVariant},
	util::Fields,
	validate::{Validate, ValidateFieldWrapper, ValidateOptions, ValidateVariant},
};

fn default_context() -> syn::Type {
	syn::parse_quote! { () }
}

fn default_crate_name() -> syn::Path {
	syn::parse_quote! { ::wary }
}

#[derive(FromDeriveInput)]
#[darling(attributes(wary))]
pub struct Options {
	ident: syn::Ident,
	generics: syn::Generics,

	/// The context type to use when validating.
	#[darling(default = "default_context")]
	context: syn::Type,
	#[darling(default = "default_crate_name", rename = "crate")]
	crate_name: syn::Path,
}

pub struct Emit {
	options: Options,
	validate: Validate,
	modify: Modify,
}

impl Emit {
	pub fn into_token_stream(self) -> TokenStream {
		match (self.validate.data, self.modify.data) {
			(ast::Data::Enum(validate), ast::Data::Enum(modify)) => EmitEnum {
				options: &self.options,
				validate,
				modify,
				validate_top: ValidateOptions {
					func: self.validate.func,
					or: self.validate.or,
					custom: self.validate.custom,
				},
				modify_top: ModifyOptions {
					func: self.modify.func,
					custom: self.modify.custom,
				},
			}
			.into_token_stream(),
			(ast::Data::Struct(validate), ast::Data::Struct(modify)) => EmitStruct {
				options: &self.options,
				validate,
				modify,
				validate_top: ValidateOptions {
					func: self.validate.func,
					or: self.validate.or,
					custom: self.validate.custom,
				},
				modify_top: ModifyOptions {
					func: self.modify.func,
					custom: self.modify.custom,
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
		let modify = Modify::from_derive_input(input)?;

		Ok(Self {
			options,
			validate,
			modify,
		})
	}
}

struct EmitEnum<'o> {
	options: &'o Options,
	validate: Vec<ValidateVariant>,
	modify: Vec<ModifyVariant>,
	validate_top: ValidateOptions,
	modify_top: ModifyOptions,
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

		let modify = self.modify.into_iter().map(|m| {
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

		let modify_top = self
			.modify_top
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
			impl #imp #crate_name::Modify for #ident #ty #wher {
				fn modify(&mut self, ctx: &Self::Context) {
					match self {
						#(
							#modify
						)*
					};

					#modify_top
				}
			}
		}
	}
}

struct EmitStruct<'o> {
	options: &'o Options,
	validate: ast::Fields<ValidateFieldWrapper>,
	modify: ast::Fields<ModifyFieldWrapper>,
	validate_top: ValidateOptions,
	modify_top: ModifyOptions,
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

		let modify = self
			.modify
			.into_iter()
			.zip(&idents)
			.map(|(m, i)| m.into_token_stream(&self.options.crate_name, i));

		let modify_top = self
			.modify_top
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
			impl #imp #crate_name::Modify for #ident #ty #wher {
				fn modify(&mut self, ctx: &Self::Context) {
					let Self { #destruct } = self;

					#(
						#modify
					)*

					#modify_top
				}
			}
		}
	}
}
