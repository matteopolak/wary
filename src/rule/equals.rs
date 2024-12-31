use super::Unset;
use crate::{Error, Validate};

#[doc(hidden)]
pub type Rule<T, O> = EqualsRule<T, O>;

pub struct EqualsRule<T, O> {
	inner: T,
	other: O,
}

impl<T> EqualsRule<T, Unset> {
	pub fn new(inner: T) -> EqualsRule<T, Unset> {
		EqualsRule {
			inner,
			other: Unset,
		}
	}
}

impl<T, O> EqualsRule<T, O> {
	pub fn other(self, other: O) -> EqualsRule<T, O> {
		EqualsRule {
			inner: self.inner,
			other,
		}
	}
}

impl<T, O> Validate for EqualsRule<T, O>
where
	T: PartialEq<O>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context) -> Result<(), Error> {
		if self.inner == self.other {
			Ok(())
		} else {
			panic!()
		}
	}
}
