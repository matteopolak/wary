use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule_<O> = EqualsRule<O>;

pub struct EqualsRule<O> {
	other: O,
}

impl EqualsRule<Unset> {
	pub fn new() -> EqualsRule<Unset> {
		EqualsRule { other: Unset }
	}

	pub fn other<O>(self, other: O) -> EqualsRule<O> {
		EqualsRule { other }
	}
}

impl<I: ?Sized, O> Rule<I> for EqualsRule<O>
where
	I: PartialEq<O>,
{
	type Context = ();

	#[inline]
	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<(), Error> {
		if *item == self.other {
			Ok(())
		} else {
			panic!()
		}
	}
}
