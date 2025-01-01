use darling::{ast, FromDeriveInput, FromField, FromVariant};

#[derive(Debug, FromField)]
#[darling(attributes(modify))]
pub struct ModifyFieldWrapper {
	pub ident: Option<syn::Ident>,
}

#[derive(Debug, FromVariant)]
pub struct ModifyVariant {
	ident: syn::Ident,
	fields: ast::Fields<ModifyFieldWrapper>,
}

#[derive(FromDeriveInput)]
#[darling(attributes(modify))]
pub struct Modify {
	pub data: ast::Data<ModifyVariant, ModifyFieldWrapper>,
}
