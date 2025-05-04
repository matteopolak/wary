//! Rule for requring a value to be empty or not empty.
//!
//! See [`RequiredRule`] for more information.

use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule<Mode> = RequiredRule<Mode>;

#[derive(Debug, thiserror::Error, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), serde(untagged))]
pub enum Error {
	#[error("value should be empty")]
	ShouldBeEmpty,
	#[error("value should not be empty")]
	CannotBeEmpty,
}

impl Error {
	#[must_use]
	pub(crate) fn code(&self) -> &'static str {
		match self {
			Self::ShouldBeEmpty => "should_be_empty",
			Self::CannotBeEmpty => "cannot_be_empty",
		}
	}

	pub(crate) fn message(&self) -> &'static str {
		match self {
			Self::ShouldBeEmpty => "value should be empty",
			Self::CannotBeEmpty => "value should not be empty",
		}
	}
}

pub struct Not;

/// Rule for requring a value to be empty or not empty.
///
/// # Example
///
/// ```
/// use wary::{Wary, Validate};
///
/// #[derive(Wary)]
/// struct Person {
///   #[validate(required)]
///   name: String,
///   #[validate(required(not))]
///   numbers: Vec<u8>,
///   #[validate(required)]
///   greeting: Option<String>,
///   #[validate(required, inner(required))]
///   nested: Option<Option<String>>,
/// }
///
/// let person = Person {
///   name: "hello".into(), // good
///   numbers: vec![1, 2, 3], // error, should be empty
///   greeting: None, // error, should not be empty
///   nested: Some(None), // error, should not be empty
/// };
///
/// assert!(person.validate(&()).is_err());
///
/// let person = Person {
///   name: "hello".into(), // good
///   numbers: vec![], // good
///   greeting: Some("hello".into()), // good
///   nested: Some(None), // good. inner(required) is applied to the nested String
/// };
///
/// assert!(person.validate(&()).is_ok());
/// ```
#[must_use]
pub struct RequiredRule<Mode> {
	mode: PhantomData<Mode>,
}

impl RequiredRule<Unset> {
	#[inline]
	pub const fn new() -> Self {
		Self { mode: PhantomData }
	}

	/// Require the value to be empty.
	///
	/// # Example
	///
	/// ```
	/// use wary::{Wary, Validate};
	///
	/// #[derive(Wary)]
	/// struct Person {
	///  #[validate(required(not))]
	///  name: String,
	/// }
	///
	/// let person = Person {
	///   name: "hello".into(), // error, should be empty
	/// };
	///
	/// assert!(person.validate(&()).is_err());
	/// ```
	#[inline]
	pub const fn not(self) -> RequiredRule<Not> {
		RequiredRule { mode: PhantomData }
	}
}

impl<I: ?Sized> crate::Rule<I> for RequiredRule<Unset>
where
	I: AsSlice,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let slice = item.as_slice();

		if slice.is_empty() {
			Err(Error::CannotBeEmpty.into())
		} else {
			Ok(())
		}
	}
}

impl<I: ?Sized> crate::Rule<I> for RequiredRule<Not>
where
	I: AsSlice,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let slice = item.as_slice();

		if slice.is_empty() {
			Ok(())
		} else {
			Err(Error::ShouldBeEmpty.into())
		}
	}
}

#[cfg(test)]
mod test {
	use super::{Not, RequiredRule};
	use crate::toolbox::test::*;

	const rule: RequiredRule<Unset> = RequiredRule::new();
	const not: RequiredRule<Not> = RequiredRule::new().not();

	#[test]
	fn test_required_rule_option() {
		assert!(rule.validate(&(), &Some(1)).is_ok());
		assert!(rule.validate(&(), &None::<i32>).is_err());

		assert!(not.validate(&(), &Some(1)).is_err());
		assert!(not.validate(&(), &None::<i32>).is_ok());
	}

	#[test]
	fn test_required_rule_slice() {
		assert!(rule.validate(&(), &[1]).is_ok());
		assert!(rule.validate(&(), &vec![1, 2, 3]).is_ok());
		assert!(rule.validate(&(), "hello").is_ok());

		assert!(rule.validate(&(), &[0; 0]).is_err());
		assert!(rule.validate(&(), &Vec::<i32>::new()).is_err());
		assert!(rule.validate(&(), "").is_err());

		assert!(not.validate(&(), &[1]).is_err());
		assert!(not.validate(&(), &vec![1, 2, 3]).is_err());
		assert!(not.validate(&(), "hello").is_err());

		assert!(not.validate(&(), &[0; 0]).is_ok());
		assert!(not.validate(&(), &Vec::<i32>::new()).is_ok());
		assert!(not.validate(&(), "").is_ok());
	}
}
