use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule<P> = SuffixRule<P>;

pub struct SuffixRule<P> {
	suffix: P,
}

impl SuffixRule<Unset> {
	pub fn new() -> Self {
		Self { suffix: Unset }
	}

	pub fn suffix<P>(self, suffix: P) -> SuffixRule<P> {
		SuffixRule { suffix }
	}
}

impl<I: ?Sized, P, O> crate::Rule<I> for SuffixRule<P>
where
	I: AsSlice<Item = O>,
	P: AsSlice<Item = O>,
	O: PartialEq,
{
	type Context = ();

	#[inline]
	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let inner = item.as_slice();
		let suffix = self.suffix.as_slice();

		if inner.ends_with(suffix) {
			Ok(())
		} else {
			panic!()
		}
	}
}
