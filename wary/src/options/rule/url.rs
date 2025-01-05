//! Rule for URL validation.
//!
//! See [`UrlRule`] for more information.

use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule = UrlRule;

#[derive(Debug, thiserror::Error, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case", tag = "code"))]
pub enum Error {
	#[error("empty host")]
	EmptyHost,
	#[error("invalid international domain name")]
	IdnaError,
	#[error("invalid port number")]
	InvalidPort,
	#[error("invalid IPv4 address")]
	InvalidIpv4Address,
	#[error("invalid IPv6 address")]
	InvalidIpv6Address,
	#[error("invalid domain character")]
	InvalidDomainCharacter,
	#[error("relative URL without a base")]
	RelativeUrlWithoutBase,
	#[error("relative URL with a cannot-be-a-base base")]
	RelativeUrlWithCannotBeABaseBase,
	#[error("a cannot-be-a-base URL doesn’t have a host to set")]
	SetHostOnCannotBeABaseUrl,
	#[error("URLs more than 4 GB are not supported")]
	Overflow,
	#[error("unknown URL error")]
	Other,
}

impl Error {
	#[must_use]
	pub(crate) fn code(&self) -> &'static str {
		match self {
			Self::EmptyHost => "empty_host",
			Self::IdnaError => "idna_error",
			Self::InvalidPort => "invalid_port",
			Self::InvalidIpv4Address => "invalid_ipv4_address",
			Self::InvalidIpv6Address => "invalid_ipv6_address",
			Self::InvalidDomainCharacter => "invalid_domain_character",
			Self::RelativeUrlWithoutBase => "relative_url_without_base",
			Self::RelativeUrlWithCannotBeABaseBase => "relative_url_with_cannot_be_a_base_base",
			Self::SetHostOnCannotBeABaseUrl => "set_host_on_cannot_be_a_base_url",
			Self::Overflow => "overflow",
			Self::Other => "other",
		}
	}

	#[cfg(feature = "alloc")]
	#[must_use]
	pub(crate) fn message(&self) -> Cow<'static, str> {
		match self {
			Self::EmptyHost => "empty host",
			Self::IdnaError => "invalid international domain name",
			Self::InvalidPort => "invalid port number",
			Self::InvalidIpv4Address => "invalid IPv4 address",
			Self::InvalidIpv6Address => "invalid IPv6 address",
			Self::InvalidDomainCharacter => "invalid domain character",
			Self::RelativeUrlWithoutBase => "relative URL without a base",
			Self::RelativeUrlWithCannotBeABaseBase => "relative URL with a cannot-be-a-base base",
			Self::SetHostOnCannotBeABaseUrl => "a cannot-be-a-base URL doesn’t have a host to set",
			Self::Overflow => "URLs more than 4 GB are not supported",
			Self::Other => "unknown URL error",
		}
		.into()
	}

	#[cfg(not(feature = "alloc"))]
	pub(crate) fn message(&self) -> &'static str {
		match self {
			Self::EmptyHost => "empty host",
			Self::IdnaError => "invalid international domain name",
			Self::InvalidPort => "invalid port number",
			Self::InvalidIpv4Address => "invalid IPv4 address",
			Self::InvalidIpv6Address => "invalid IPv6 address",
			Self::InvalidDomainCharacter => "invalid domain character",
			Self::RelativeUrlWithoutBase => "relative URL without a base",
			Self::RelativeUrlWithCannotBeABaseBase => "relative URL with a cannot-be-a-base base",
			Self::SetHostOnCannotBeABaseUrl => "a cannot-be-a-base URL doesn’t have a host to set",
			Self::Overflow => "URLs more than 4 GB are not supported",
			Self::Other => "unknown URL error",
		}
	}
}

impl From<url::ParseError> for Error {
	fn from(value: url::ParseError) -> Self {
		match value {
			url::ParseError::EmptyHost => Self::EmptyHost,
			url::ParseError::IdnaError => Self::IdnaError,
			url::ParseError::InvalidPort => Self::InvalidPort,
			url::ParseError::InvalidIpv4Address => Self::InvalidIpv4Address,
			url::ParseError::InvalidIpv6Address => Self::InvalidIpv6Address,
			url::ParseError::InvalidDomainCharacter => Self::InvalidDomainCharacter,
			url::ParseError::RelativeUrlWithoutBase => Self::RelativeUrlWithoutBase,
			url::ParseError::RelativeUrlWithCannotBeABaseBase => Self::RelativeUrlWithCannotBeABaseBase,
			url::ParseError::SetHostOnCannotBeABaseUrl => Self::SetHostOnCannotBeABaseUrl,
			url::ParseError::Overflow => Self::Overflow,
			_ => Self::Other,
		}
	}
}

/// Rule for URL validation.
///
/// # Example
///
/// ```
/// use wary::{Wary, Validate};
///
/// #[derive(Wary)]
/// struct Image {
///   label: String,
///   #[validate(url)]
///   src: String,
/// }
///
/// let image = Image {
///   label: "My cat".into(),
///   src: "https://example.com/cat.jpg".into(),
/// };
///
/// assert!(image.validate(&()).is_ok());
///
/// let image = Image {
///   label: "My cat".into(),
///   src: "hello".into(),
/// };
///
/// assert!(image.validate(&()).is_err());
/// ```
pub struct UrlRule;

impl UrlRule {
	#[must_use]
	#[inline]
	pub const fn new() -> Self {
		Self
	}
}

impl<I: ?Sized> crate::Rule<I> for UrlRule
where
	I: AsRef<str>,
{
	type Context = ();

	#[inline]
	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		url::Url::parse(item.as_ref()).map_err(Error::from)?;
		Ok(())
	}
}

#[cfg(test)]
mod test {
	use super::UrlRule;
	use crate::toolbox::test::*;

	const rule: UrlRule = UrlRule::new();

	#[test]
	fn test_url() {
		assert!(rule.validate(&(), "https://example.com").is_ok());
		assert!(rule.validate(&(), "hello").is_err());
	}
}
