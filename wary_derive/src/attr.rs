fn ungroup(mut ty: &syn::Type) -> &syn::Type {
	while let syn::Type::Group(group) = ty {
		ty = &group.elem;
	}
	ty
}

pub fn extract_option_path(ty: &syn::Type) -> Option<syn::Path> {
	let path = match ungroup(ty) {
		syn::Type::Path(ty) => &ty.path,
		_ => {
			return None;
		}
	};
	let seg = match path.segments.last() {
		Some(seg) => seg,
		None => {
			return None;
		}
	};
	let args = match &seg.arguments {
		syn::PathArguments::AngleBracketed(bracketed) => &bracketed.args,
		_ => {
			return None;
		}
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
