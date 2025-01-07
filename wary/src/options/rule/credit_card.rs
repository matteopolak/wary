//! Rule for email address validation.
//!
//! It is recommended to instead parse directly into an
//! [`EmailAddress`][email] if you need to parse it afterwards anyway. Other
//! validators that accept string-like values such as [`ascii`][ascii],
//! [`length`][length], [`contains`][contains], etc. can still be used with
//! an [`EmailAddress`][email]!
//!
//! See [`CreditCardRule`] for more information.
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
	#[error("invalid character in PAN")]
	InvalidFormat,
	#[error("unknown credit card type")]
	UnknownType,
	#[error("invalid PAN length")]
	InvalidLength,
	#[error("invalid luhn checksum")]
	InvalidLuhn,
}

impl Error {
	#[must_use]
	pub(crate) fn code(&self) -> &'static str {
		match self {
			Self::InvalidFormat => "invalid_card_format",
			Self::UnknownType => "unknown_card_type",
			Self::InvalidLength => "invalid_card_length",
			Self::InvalidLuhn => "invalid_card_luhn",
		}
	}

	#[cfg(feature = "alloc")]
	#[must_use]
	pub(crate) fn message(&self) -> Cow<'static, str> {
		match self {
			Self::InvalidFormat => "invalid character in PAN",
			Self::UnknownType => "unknown credit card type",
			Self::InvalidLength => "invalid PAN length",
			Self::InvalidLuhn => "invalid luhn checksum",
		}
		.into()
	}

	#[cfg(not(feature = "alloc"))]
	pub(crate) fn message(&self) -> &'static str {
		match self {
			Self::InvalidFormat => "invalid character in PAN",
			Self::UnknownType => "unknown credit card type",
			Self::InvalidLength => "invalid PAN length",
			Self::InvalidLuhn => "invalid luhn checksum",
		}
	}
}

impl From<creditcard::Error> for Error {
	fn from(value: creditcard::Error) -> Self {
		use creditcard::Error::*;

		match value {
			InvalidFormat => Self::InvalidFormat,
			UnknownType => Self::UnknownType,
			InvalidLength => Self::InvalidLength,
			InvalidLuhn => Self::InvalidLuhn,
		}
	}
}

#[doc(hidden)]
pub type Rule = CreditCardRule;

/// Rule for email validation.
///
/// # Example
///
/// ```
/// use wary::{Wary, Validate};
///
/// #[derive(Wary)]
/// struct Person {
///   #[validate(credit_card)]
///   pan: String,
/// }
///
/// let person = Person {
///   pan: "4111111111111111".into(),
/// };
///
/// assert!(person.validate(&()).is_ok());
///
/// let person = Person {
///   pan: "4111111111111112".into(),
/// };
///
/// assert!(person.validate(&()).is_err());
/// ```
#[must_use]
pub struct CreditCardRule;

impl CreditCardRule {
	#[inline]
	pub const fn new() -> Self {
		Self
	}
}

impl<I: ?Sized> crate::Rule<I> for CreditCardRule
where
	I: AsRef<str>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let email = item.as_ref();

		creditcard::CreditCard::from_str(email).map_err(Error::from)?;
		Ok(())
	}
}

#[cfg(test)]
mod test {
	use crate::toolbox::test::*;

	#[derive(Wary)]
	#[wary(crate = "crate")]
	struct Person {
		#[validate(credit_card)]
		pan: String,
	}

	#[test]
	fn test_credit_card_rule() {
		let person = Person {
			pan: "4111111111111111".into(),
		};

		assert!(person.validate(&()).is_ok());

		let person = Person {
			pan: "4111111111111112".into(),
		};

		assert!(person.validate(&()).is_err());
	}
}
