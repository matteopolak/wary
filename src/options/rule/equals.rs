use core::fmt;

use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule<O, Mode> = EqualsRule<O, Mode>;

pub struct Not;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("expected to equal {0:?}")]
	ShouldEqual(Box<dyn fmt::Debug>),
	#[error("expected to not equal {0:?}")]
	ShouldNotEqual(Box<dyn fmt::Debug>),
}

pub struct EqualsRule<O, Mode> {
	other: O,
	mode: PhantomData<Mode>,
}

impl<M> EqualsRule<Unset, M> {
	#[must_use]
	pub fn new() -> EqualsRule<Unset, Unset> {
		EqualsRule {
			other: Unset,
			mode: PhantomData,
		}
	}

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
	pub fn not(self) -> EqualsRule<O, Not> {
		EqualsRule {
			other: self.other,
			mode: PhantomData,
		}
	}
}

impl<I: ?Sized, O> crate::Rule<I> for EqualsRule<O, Unset>
where
	O: fmt::Debug + Copy + 'static,
	I: PartialEq<O>,
{
	type Context = ();

	#[inline]
	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		if *item == self.other {
			Ok(())
		} else {
			Err(Error::ShouldEqual(Box::new(self.other)).into())
		}
	}
}

impl<I: ?Sized, O> crate::Rule<I> for EqualsRule<O, Not>
where
	O: fmt::Debug + Copy + 'static,
	I: PartialEq<O>,
{
	type Context = ();

	#[inline]
	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		if *item == self.other {
			Err(Error::ShouldNotEqual(Box::new(self.other)).into())
		} else {
			Ok(())
		}
	}
}
