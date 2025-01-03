use core::fmt;

use crate::{options::DebugDisplay, toolbox::rule::*};

#[doc(hidden)]
pub type Rule<O, Mode> = EqualsRule<O, Mode>;

pub struct Not;

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
	#[error("expected to equal {0}")]
	ShouldEqual(String),
	#[error("expected to not equal {0}")]
	ShouldNotEqual(String),
}

#[must_use]
pub struct EqualsRule<O, Mode> {
	other: O,
	mode: PhantomData<Mode>,
}

impl EqualsRule<Unset, Unset> {
	#[inline]
	pub fn new() -> EqualsRule<Unset, Unset> {
		EqualsRule {
			other: Unset,
			mode: PhantomData,
		}
	}
}

impl<M> EqualsRule<Unset, M> {
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
	for<'i> &'i I: PartialEq<O>,
{
	type Context = ();

	#[inline]
	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		if item == self.other {
			Ok(())
		} else {
			Err(Error::ShouldEqual(DebugDisplay(&self.other).to_string()).into())
		}
	}
}

impl<I: ?Sized, O> crate::Rule<I> for EqualsRule<O, Not>
where
	O: fmt::Debug + Copy + 'static,
	for<'i> &'i I: PartialEq<O>,
{
	type Context = ();

	#[inline]
	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		if item == self.other {
			Err(Error::ShouldNotEqual(DebugDisplay(&self.other).to_string()).into())
		} else {
			Ok(())
		}
	}
}

#[cfg(test)]
mod test {
	use std::borrow::Cow;

	use crate::toolbox::test::*;

	#[test]
	fn test_equals_rule() {
		#[derive(Wary)]
		#[wary(crate = "crate")]
		struct Person<'name> {
			#[validate(equals(other = "hello"))]
			name: Cow<'name, str>,
		}

		let person = Person {
			name: Cow::Borrowed("hello"),
		};

		assert!(person.validate(&()).is_ok());

		let person = Person {
			name: Cow::Borrowed("world"),
		};

		assert!(person.validate(&()).is_err());
	}
}
