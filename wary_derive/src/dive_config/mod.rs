#[derive(Debug, Default, Clone)]
pub enum DiveConfig {
	#[default]
    None,
    Default,
    Custom(syn::Expr),
}

pub fn parse_dive(meta: &syn::Meta) -> darling::Result<DiveConfig> {
    match meta {
        syn::Meta::Path(_) => Ok(DiveConfig::Default),
        
        syn::Meta::List(meta_list) => {
            let tokens = &meta_list.tokens;
            let expr = syn::parse2::<syn::Expr>(tokens.clone())
                .map_err(|e| darling::Error::custom(e.to_string()).with_span(tokens))?;
            Ok(DiveConfig::Custom(expr))
        }
        
        syn::Meta::NameValue(nv) => {
            Err(darling::Error::unsupported_format("name-value").with_span(nv))
        }
    }
}

pub fn parse_dive_opt(meta: &syn::Meta) -> darling::Result<Option<DiveConfig>> {
    parse_dive(meta).map(Some)
}
