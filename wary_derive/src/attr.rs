fn ungroup(mut ty: &syn::Type) -> &syn::Type {
	while let syn::Type::Group(group) = ty {
		ty = &group.elem;
	}
	ty
}

pub fn extract_str(expr: &syn::Expr) -> Option<String> {
	match expr {
		syn::Expr::Lit(lit) => match &lit.lit {
			syn::Lit::Str(str_) => Some(str_.value()),
			_ => None,
		},
		_ => None,
	}
}

pub fn extract_option_path(ty: &syn::Type) -> Option<syn::Path> {
	let syn::Type::Path(syn::TypePath { path, .. }) = ungroup(ty) else {
		return None;
	};
	let seg = path.segments.last()?;
	let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments { args, .. }) =
		&seg.arguments
	else {
		return None;
	};

	if seg.ident == "Option" && args.len() == 1 && matches!(args[0], syn::GenericArgument::Type(..)) {
		let mut path = path.clone();
		let last = path.segments.last_mut().unwrap();

		last.arguments = syn::PathArguments::None;

		Some(path)
	} else {
		None
	}
}
