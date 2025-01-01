use darling::{ast, FromDeriveInput};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::{
	modify::{Modify, ModifyFieldWrapper, ModifyVariant},
	util::Fields,
	validate::{Validate, ValidateFieldWrapper, ValidateVariant},
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
		let inner = match (self.validate.data, self.modify.data) {
			(ast::Data::Enum(validate), ast::Data::Enum(modify)) => EmitEnum {
				options: &self.options,
				validate,
				modify,
			}
			.into_token_stream(),
			(ast::Data::Struct(validate), ast::Data::Struct(modify)) => EmitStruct {
				options: &self.options,
				validate,
				modify,
			}
			.into_token_stream(),
			_ => unimplemented!(),
		};

		let (imp, ty, wher) = self.options.generics.split_for_impl();
		let crate_name = &self.options.crate_name;
		let context = &self.options.context;
		let ident = &self.options.ident;

		quote! {
			impl #imp #crate_name::Validate for #ident #ty #wher {
				type Context = #context;

				fn validate(&self, ctx: &Self::Context) -> Result<(), #crate_name::Error> {
					#inner

					Ok(())
				}
			}
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
}

impl EmitEnum<'_> {
	fn into_token_stream(self) -> TokenStream {
		let variants = self.validate.into_iter().zip(self.modify).map(|(v, m)| {
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

		quote! {
			match self {
				#(
					#variants
				)*
			};
		}
	}
}

struct EmitStruct<'o> {
	options: &'o Options,
	validate: ast::Fields<ValidateFieldWrapper>,
	modify: ast::Fields<ModifyFieldWrapper>,
}

impl EmitStruct<'_> {
	fn into_token_stream(self) -> TokenStream {
		let destruct = Fields(&self.validate).destruct();

		let fields = self
			.validate
			.into_iter()
			.zip(self.modify)
			.enumerate()
			.map(|(i, (v, m))| {
				let ident = v.ident.clone().unwrap_or_else(|| format_ident!("_{i}"));

				v.into_token_stream(&self.options.crate_name, &ident)
			});

		quote! {
			let Self { #destruct } = self;

			#(
				#fields
			)*
		}
	}
}
