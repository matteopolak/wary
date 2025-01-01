use super::{contains::ToSlice, Unset};
use crate::{Error, Validate};

#[doc(hidden)]
pub type Rule<T, P> = SuffixRule<T, P>;

pub struct SuffixRule<T, P> {
	inner: T,
	suffix: P,
}

impl<T> SuffixRule<T, Unset> {
	pub fn new(inner: T) -> Self {
		Self {
			inner,
			suffix: Unset,
		}
	}
}

impl<T> SuffixRule<T, Unset> {
	pub fn suffix<P>(self, suffix: P) -> SuffixRule<T, P> {
		SuffixRule {
			inner: self.inner,
			suffix,
		}
	}
}

impl<T, P, I> Validate for SuffixRule<T, P>
where
	T: ToSlice<Item = I>,
	P: ToSlice<Item = I>,
	I: PartialEq,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context) -> Result<(), Error> {
		let inner = self.inner.to_slice();
		let suffix = self.suffix.to_slice();

		if inner.ends_with(suffix) {
			Ok(())
		} else {
			panic!()
		}
	}
}
