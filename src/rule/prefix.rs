use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule_<P> = PrefixRule<P>;

pub struct PrefixRule<P> {
	prefix: P,
}

impl PrefixRule<Unset> {
	pub fn new() -> Self {
		Self { prefix: Unset }
	}

	pub fn prefix<P>(self, prefix: P) -> PrefixRule<P> {
		PrefixRule { prefix }
	}
}

impl<I: ?Sized, P, O> Rule<I> for PrefixRule<P>
where
	I: AsSlice<Item = O>,
	P: AsSlice<Item = O>,
	O: PartialEq,
{
	type Context = ();

	#[inline]
	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<(), Error> {
		let inner = item.as_slice();
		let prefix = self.prefix.as_slice();

		if inner.starts_with(prefix) {
			Ok(())
		} else {
			panic!()
		}
	}
}
