//! Rule for UUID validation.
//!
//! See [`UuidRule`] for more information.

use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule = UuidRule;

#[derive(Debug, thiserror::Error, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), serde(untagged))]
pub enum Error {
	#[error("expected valid UUID")]
	Uuid,
}

impl Error {
	#[must_use]
	pub(crate) fn code(&self) -> &'static str {
		match self {
			Self::Uuid => "uuid",
		}
	}

	#[cfg(feature = "alloc")]
	#[must_use]
	pub(crate) fn message(&self) -> Cow<'static, str> {
		match self {
			Self::Uuid => "expected valid UUID",
		}
		.into()
	}

	#[cfg(not(feature = "alloc"))]
	pub(crate) fn message(&self) -> &'static str {
		match self {
			Self::Uuid => "expected valid UUID",
		}
	}
}

/// Rule for UUID validation.
///
/// # Example
///
/// ```
/// use wary::{Wary, Validate};
///
/// #[derive(Wary)]
/// struct Person {
///   #[validate(uuid)]
///   id: String,
/// }
///
/// let person = Person {
///   id: "550e8400-e29b-41d4-a716-446655440000".into(),
/// };
///
/// assert!(person.validate(&()).is_ok());
///
/// let person = Person {
///   id: "hello".into(),
/// };
///
/// assert!(person.validate(&()).is_err());
/// ```
pub struct UuidRule;

impl UuidRule {
	#[must_use]
	pub const fn new() -> Self {
		Self
	}
}

impl<I: ?Sized> crate::Rule<I> for UuidRule
where
	I: AsRef<str>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let string = item.as_ref();

		uuid::Uuid::parse_str(string).map_err(|_| Error::Uuid)?;

		Ok(())
	}
}

#[cfg(test)]
mod test {
	use crate::toolbox::test::*;

	#[derive(Wary)]
	#[wary(crate = "crate")]
	struct Person<'name> {
		#[validate(uuid)]
		id: Cow<'name, str>,
	}

	#[test]
	fn test_uuid_rule() {
		let person = Person {
			id: Cow::Borrowed("550e8400-e29b-41d4-a716-446655440000"),
		};

		assert!(person.validate(&()).is_ok());

		let person = Person {
			id: Cow::Borrowed("hello"),
		};

		assert!(person.validate(&()).is_err());
	}
}
