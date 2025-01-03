use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule<P, Mode, Kind> = PrefixRule<P, Mode, Kind>;

pub struct Str;
pub struct Slice;

pub struct Not;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("expected string to start with \"{0}\"")]
	ShouldStartWith(&'static str),
	#[error("expected string to not start with \"{0}\"")]
	ShouldNotStartWith(&'static str),
	#[error("expected slice to start with")]
	ShouldStartWithSlice,
	#[error("expected slice to not start with")]
	ShouldNotStartWithSlice,
}

pub struct PrefixRule<P, Mode, Kind> {
	prefix: P,
	mode: PhantomData<Mode>,
	kind: PhantomData<Kind>,
}

impl<M> PrefixRule<Unset, M, Unset> {
	#[must_use]
	pub fn new() -> Self {
		Self {
			prefix: Unset,
			mode: PhantomData,
			kind: PhantomData,
		}
	}

	#[must_use]
	pub fn str(self, prefix: &'static str) -> PrefixRule<&'static str, M, Str> {
		PrefixRule {
			prefix,
			mode: PhantomData,
			kind: PhantomData,
		}
	}

	pub fn slice<P>(self, prefix: P) -> PrefixRule<P, M, Slice> {
		PrefixRule {
			prefix,
			mode: PhantomData,
			kind: PhantomData,
		}
	}
}

impl<P, K> PrefixRule<P, Unset, K> {
	pub fn not(self) -> PrefixRule<P, Not, K> {
		PrefixRule {
			prefix: self.prefix,
			mode: PhantomData,
			kind: PhantomData,
		}
	}
}

impl<I: ?Sized, P, O> crate::Rule<I> for PrefixRule<P, Unset, Slice>
where
	I: AsSlice<Item = O>,
	P: AsSlice<Item = O>,
	O: PartialEq,
{
	type Context = ();

	#[inline]
	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let inner = item.as_slice();
		let prefix = self.prefix.as_slice();

		if inner.starts_with(prefix) {
			Ok(())
		} else {
			Err(Error::ShouldStartWithSlice.into())
		}
	}
}

impl<I: ?Sized, P, O> crate::Rule<I> for PrefixRule<P, Not, Slice>
where
	I: AsSlice<Item = O>,
	P: AsSlice<Item = O>,
	O: PartialEq,
{
	type Context = ();

	#[inline]
	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let inner = item.as_slice();
		let prefix = self.prefix.as_slice();

		if inner.starts_with(prefix) {
			Err(Error::ShouldNotStartWithSlice.into())
		} else {
			Ok(())
		}
	}
}

impl<I: ?Sized> crate::Rule<I> for PrefixRule<&'static str, Unset, Str>
where
	I: AsRef<str>,
{
	type Context = ();

	#[inline]
	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let inner = item.as_ref();
		let prefix = self.prefix;

		if inner.starts_with(prefix) {
			Ok(())
		} else {
			Err(Error::ShouldStartWith(self.prefix).into())
		}
	}
}

impl<I: ?Sized> crate::Rule<I> for PrefixRule<&'static str, Not, Str>
where
	I: AsRef<str>,
{
	type Context = ();

	#[inline]
	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let inner = item.as_ref();
		let prefix = self.prefix;

		if inner.starts_with(prefix) {
			Err(Error::ShouldNotStartWith(self.prefix).into())
		} else {
			Ok(())
		}
	}
}
