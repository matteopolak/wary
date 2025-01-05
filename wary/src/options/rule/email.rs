//! Rule for email address validation.
//!
//! It is recommended to instead parse directly into an
//! [`EmailAddress`][email] if you need to parse it afterwards anyway. Other
//! validators that accept string-like values such as [`ascii`][ascii],
//! [`length`][length], [`contains`][contains], etc. can still be used with
//! an [`EmailAddress`][email]!
//!
//! See [`EmailRule`] for more information.
//!
//! [ascii]: crate::rule::ascii
//! [contains]: crate::rule::contains
//! [length]: crate::rule::length
//!
//! [email]: email_address::EmailAddress

use core::str::FromStr;

use crate::toolbox::rule::*;

#[derive(Debug, thiserror::Error, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case", tag = "code"))]
pub enum Error {
	#[error("invalid character")]
	InvalidCharacter,
	#[error("missing separator")]
	MissingSeparator,
	#[error("local-part is empty")]
	LocalPartEmpty,
	#[error("local-part is too long")]
	LocalPartTooLong,
	#[error("domain is empty")]
	DomainEmpty,
	#[error("domain is too long")]
	DomainTooLong,
	#[error("sub-domain is empty")]
	SubDomainEmpty,
	#[error("sub-domain is too long")]
	SubDomainTooLong,
	#[error("too few sub-domains")]
	DomainTooFew,
	#[error("invalid separator")]
	DomainInvalidSeparator,
	#[error("unbalanced quotes in local-part")]
	UnbalancedQuotes,
	#[error("invalid comment")]
	InvalidComment,
	#[error("invalid IP address")]
	InvalidIPAddress,
	#[error("unsupported domain literal")]
	UnsupportedDomainLiteral,
	#[error("unsupported display name")]
	UnsupportedDisplayName,
	#[error("missing display name")]
	MissingDisplayName,
	#[error("missing end bracket")]
	MissingEndBracket,
}

impl Error {
	#[must_use]
	pub(crate) fn code(&self) -> &'static str {
		match self {
			Self::InvalidCharacter => "invalid_character",
			Self::MissingSeparator => "missing_separator",
			Self::LocalPartEmpty => "local_part_empty",
			Self::LocalPartTooLong => "local_part_too_long",
			Self::DomainEmpty => "domain_empty",
			Self::DomainTooLong => "domain_too_long",
			Self::SubDomainEmpty => "sub_domain_empty",
			Self::SubDomainTooLong => "sub_domain_too_long",
			Self::DomainTooFew => "domain_too_few",
			Self::DomainInvalidSeparator => "domain_invalid_separator",
			Self::UnbalancedQuotes => "unbalanced_quotes",
			Self::InvalidComment => "invalid_comment",
			Self::InvalidIPAddress => "invalid_ip_address",
			Self::UnsupportedDomainLiteral => "unsupported_domain_literal",
			Self::UnsupportedDisplayName => "unsupported_display_name",
			Self::MissingDisplayName => "missing_display_name",
			Self::MissingEndBracket => "missing_end_bracket",
		}
	}

	#[cfg(feature = "alloc")]
	#[must_use]
	pub(crate) fn message(&self) -> Cow<'static, str> {
		match self {
			Self::InvalidCharacter => "invalid character",
			Self::MissingSeparator => "missing separator",
			Self::LocalPartEmpty => "local-part is empty",
			Self::LocalPartTooLong => "local-part is too long",
			Self::DomainEmpty => "domain is empty",
			Self::DomainTooLong => "domain is too long",
			Self::SubDomainEmpty => "sub-domain is empty",
			Self::SubDomainTooLong => "sub-domain is too long",
			Self::DomainTooFew => "too few sub-domains",
			Self::DomainInvalidSeparator => "invalid separator",
			Self::UnbalancedQuotes => "unbalanced quotes in local-part",
			Self::InvalidComment => "invalid comment",
			Self::InvalidIPAddress => "invalid IP address",
			Self::UnsupportedDomainLiteral => "unsupported domain literal",
			Self::UnsupportedDisplayName => "unsupported display name",
			Self::MissingDisplayName => "missing display name",
			Self::MissingEndBracket => "missing end bracket",
		}
		.into()
	}

	#[cfg(not(feature = "alloc"))]
	pub(crate) fn message(&self) -> &'static str {
		match self {
			Self::InvalidCharacter => "invalid character",
			Self::MissingSeparator => "missing separator",
			Self::LocalPartEmpty => "local-part is empty",
			Self::LocalPartTooLong => "local-part is too long",
			Self::DomainEmpty => "domain is empty",
			Self::DomainTooLong => "domain is too long",
			Self::SubDomainEmpty => "sub-domain is empty",
			Self::SubDomainTooLong => "sub-domain is too long",
			Self::DomainTooFew => "too few sub-domains",
			Self::DomainInvalidSeparator => "invalid separator",
			Self::UnbalancedQuotes => "unbalanced quotes in local-part",
			Self::InvalidComment => "invalid comment",
			Self::InvalidIPAddress => "invalid IP address",
			Self::UnsupportedDomainLiteral => "unsupported domain literal",
			Self::UnsupportedDisplayName => "unsupported display name",
			Self::MissingDisplayName => "missing display name",
			Self::MissingEndBracket => "missing end bracket",
		}
	}
}

impl From<email_address::Error> for Error {
	fn from(value: email_address::Error) -> Self {
		use email_address::Error::*;

		match value {
			InvalidCharacter => Self::InvalidCharacter,
			MissingSeparator => Self::MissingSeparator,
			LocalPartEmpty => Self::LocalPartEmpty,
			LocalPartTooLong => Self::LocalPartTooLong,
			DomainEmpty => Self::DomainEmpty,
			DomainTooLong => Self::DomainTooLong,
			SubDomainEmpty => Self::SubDomainEmpty,
			SubDomainTooLong => Self::SubDomainTooLong,
			DomainTooFew => Self::DomainTooFew,
			DomainInvalidSeparator => Self::DomainInvalidSeparator,
			UnbalancedQuotes => Self::UnbalancedQuotes,
			InvalidComment => Self::InvalidComment,
			InvalidIPAddress => Self::InvalidIPAddress,
			UnsupportedDomainLiteral => Self::UnsupportedDomainLiteral,
			UnsupportedDisplayName => Self::UnsupportedDisplayName,
			MissingDisplayName => Self::MissingDisplayName,
			MissingEndBracket => Self::MissingEndBracket,
		}
	}
}

#[doc(hidden)]
pub type Rule = EmailRule;

/// Rule for email validation.
///
/// # Example
///
/// ```
/// use wary::{Wary, Validate};
///
/// #[derive(Wary)]
/// struct Person {
///   #[validate(email)]
///   email: String,
/// }
///
/// let person = Person {
///   email: "hello@email.com".into(),
/// };
///
/// assert!(person.validate(&()).is_ok());
///
/// let person = Person {
///   email: "hello".into(),
/// };
///
/// assert!(person.validate(&()).is_err());
/// ```
#[must_use]
pub struct EmailRule;

impl EmailRule {
	#[inline]
	pub const fn new() -> Self {
		Self
	}
}

impl<I: ?Sized> crate::Rule<I> for EmailRule
where
	I: AsRef<str>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let email = item.as_ref();

		// TODO: Look into avoiding the allocation
		email_address::EmailAddress::from_str(email).map_err(Error::from)?;
		Ok(())
	}
}

#[cfg(test)]
mod test {
	use crate::toolbox::test::*;

	#[derive(Wary)]
	#[wary(crate = "crate")]
	struct Person {
		#[validate(email)]
		email: String,
	}

	#[test]
	fn test_email_rule() {
		let person = Person {
			email: "hello@email.com".into(),
		};

		assert!(person.validate(&()).is_ok());

		let person = Person {
			email: "hello".into(),
		};

		assert!(person.validate(&()).is_err());
	}
}
