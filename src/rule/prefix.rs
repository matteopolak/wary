use crate::{Error, Validate};

use super::{contains::ToSlice, Unset};

#[doc(hidden)]
pub type Rule<T, P> = PrefixRule<T, P>;

pub struct PrefixRule<T, P> {
	inner: T,
	prefix: P,
}

impl<T> PrefixRule<T, Unset> {
	pub fn new(inner: T) -> Self {
		Self { inner, prefix: Unset }
	}
}

impl<T> PrefixRule<T, Unset> {
	pub fn prefix<P>(self, prefix: P) -> PrefixRule<T, P> {
		PrefixRule {
			inner: self.inner,
			prefix,
		}
	}
}

impl<T, P, I> Validate for PrefixRule<T, P>
where
	T: ToSlice<Item = I>,
	P: ToSlice<Item = I>,
	I: PartialEq
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context) -> Result<(), Error> {
		let inner = self.inner.to_slice();
		let prefix = self.prefix.to_slice();

		if inner.starts_with(prefix) {
			Ok(())
		} else {
		panic!()
		}
	}
}
