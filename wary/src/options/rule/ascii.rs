//! Rule for ASCII validation.
//!
//! See [`AsciiRule`] for more information.

use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule = AsciiRule;

#[derive(Debug, thiserror::Error, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case", tag = "code"))]
pub enum Error {
	#[error("expected ascii")]
	Ascii,
}

impl Error {
	#[must_use]
	pub(crate) fn code(&self) -> &'static str {
		match self {
			Self::Ascii => "ascii",
		}
	}

	#[cfg(feature = "alloc")]
	#[must_use]
	pub(crate) fn message(&self) -> Cow<'static, str> {
		match self {
			Self::Ascii => "expected ASCII",
		}
		.into()
	}

	#[cfg(not(feature = "alloc"))]
	pub(crate) fn message(&self) -> &'static str {
		match self {
			Self::Ascii => "expected ASCII",
		}
	}
}

/// Rule for ASCII validation.
///
/// # Example
///
/// ```
/// use wary::{Wary, Validate};
///
/// #[derive(Wary)]
/// struct Person {
///   #[validate(ascii)]
///   name: String,
/// }
///
/// let person = Person {
///   name: "hello".into(),
/// };
///
/// assert!(person.validate(&()).is_ok());
///
/// let person = Person {
///   name: "hello 😃".into(),
/// };
///
/// assert!(person.validate(&()).is_err());
/// ```
pub struct AsciiRule;

impl AsciiRule {
	#[must_use]
	pub const fn new() -> Self {
		Self
	}
}

impl<I: ?Sized> crate::Rule<I> for AsciiRule
where
	I: AsRef<str>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let string = item.as_ref();

		if string.is_ascii() {
			Ok(())
		} else {
			Err(Error::Ascii.into())
		}
	}
}

#[cfg(test)]
mod test {
	use crate::toolbox::test::*;

	#[derive(Wary)]
	#[wary(crate = "crate")]
	struct Person<'name> {
		#[validate(ascii)]
		name: Cow<'name, str>,
	}

	#[test]
	fn test_ascii_rule() {
		let person = Person {
			name: Cow::Borrowed("Hello"),
		};

		assert!(person.validate(&()).is_ok());

		let person = Person {
			name: Cow::Borrowed("hello world 😃"),
		};

		assert!(person.validate(&()).is_err());
	}
}
