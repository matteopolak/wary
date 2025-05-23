//! Rule for equality validation.
//!
//! See [`EqualsRule`] for more information.

use core::fmt;

use crate::{
	options::{DebugDisplay, ItemSlice},
	toolbox::rule::*,
};

#[doc(hidden)]
pub type Rule<O, Mode> = EqualsRule<O, Mode>;

pub struct Not;

#[derive(Debug, thiserror::Error, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), serde(untagged))]
pub enum Error {
	#[error("expected to equal")]
	ShouldEqual { value: ItemSlice },
	#[error("expected to not equal")]
	ShouldNotEqual { value: ItemSlice },
}

impl Error {
	#[must_use]
	pub(crate) fn code(&self) -> &'static str {
		match self {
			Self::ShouldEqual { .. } => "should_equal",
			Self::ShouldNotEqual { .. } => "should_not_equal",
		}
	}

	#[cfg(feature = "alloc")]
	#[must_use]
	pub(crate) fn message(&self) -> Cow<'static, str> {
		match self {
			Self::ShouldEqual { value } => format!("expected to equal {value:?}"),
			Self::ShouldNotEqual { value } => format!("expected to not equal {value:?}"),
		}
		.into()
	}

	#[cfg(not(feature = "alloc"))]
	pub(crate) fn message(&self) -> &'static str {
		match self {
			Self::ShouldEqual { .. } => "expected to equal",
			Self::ShouldNotEqual { .. } => "expected to not equal",
		}
	}
}

/// Rule for equality validation.
///
/// # Example
///
/// ```
/// use wary::{Wary, Validate};
///
/// #[derive(Wary)]
/// struct Person {
///   #[validate(equals(other = "hello"))]
///   name: String,
///   #[validate(equals(not, other = "world"))]
///   greeting: String,
///   #[validate(equals(other = 42))]
///   age: u8,
/// }
///
/// let person = Person {
///   name: "hello".into(),
///   greeting: "hello".into(),
///   age: 42,
/// };
///
/// assert!(person.validate(&()).is_ok());
///
/// let person = Person {
///   name: "world".into(),
///   greeting: "world".into(),
///   age: 41,
/// };
///
/// assert!(person.validate(&()).is_err());
/// ```
#[must_use]
pub struct EqualsRule<O, Mode> {
	other: O,
	mode: PhantomData<Mode>,
}

impl EqualsRule<Unset, Unset> {
	#[inline]
	pub const fn new() -> EqualsRule<Unset, Unset> {
		EqualsRule {
			other: Unset,
			mode: PhantomData,
		}
	}
}

impl<M> EqualsRule<Unset, M> {
	/// Set the value to compare against.
	#[inline]
	pub fn other<O>(self, other: O) -> EqualsRule<O, M>
	where
		O: fmt::Debug + Copy + 'static,
	{
		EqualsRule {
			other,
			mode: PhantomData,
		}
	}
}

impl<O> EqualsRule<O, Unset> {
	/// Inverts the rule.
	#[inline]
	pub fn not(self) -> EqualsRule<O, Not> {
		EqualsRule {
			other: self.other,
			mode: PhantomData,
		}
	}
}

impl<I: ?Sized, O> crate::Rule<I> for EqualsRule<O, Unset>
where
	O: fmt::Debug,
	for<'i> &'i I: PartialEq<&'i O>,
{
	type Context = ();

	#[inline]
	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		if item == &self.other {
			Ok(())
		} else {
			Err(
				Error::ShouldEqual {
					value: DebugDisplay(&self.other).to_string(),
				}
				.into(),
			)
		}
	}
}

impl<I: ?Sized, O> crate::Rule<I> for EqualsRule<O, Not>
where
	O: fmt::Debug + Copy + 'static,
	for<'i> &'i I: PartialEq<&'i O>,
{
	type Context = ();

	#[inline]
	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		if item == &self.other {
			Err(
				Error::ShouldNotEqual {
					value: DebugDisplay(&self.other).to_string(),
				}
				.into(),
			)
		} else {
			Ok(())
		}
	}
}

#[cfg(test)]
mod test {
	use crate::toolbox::test::*;

	#[test]
	fn test_equals_rule() {
		#[derive(Wary)]
		#[wary(crate = "crate")]
		struct Person<'name> {
			#[validate(equals(other = "hello"))]
			name: Cow<'name, str>,
			#[validate(equals(other = 42))]
			age: u8,
		}

		let person = Person {
			name: Cow::Borrowed("hello"),
			age: 42,
		};

		assert!(person.validate(&()).is_ok());

		let person = Person {
			name: Cow::Borrowed("world"),
			age: 41,
		};

		assert!(person.validate(&()).is_err());
	}
}
