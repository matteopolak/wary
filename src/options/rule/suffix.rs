use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule<S, Mode, Kind> = SuffixRule<S, Mode, Kind>;

pub struct Str;
pub struct Slice;

pub struct Not;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("expected string to end with \"{0}\"")]
	ShouldEndWith(&'static str),
	#[error("expected string to not end with \"{0}\"")]
	ShouldNotEndWith(&'static str),
	#[error("expected slice to end with")]
	ShouldEndWithSlice,
	#[error("expected slice to not end with")]
	ShouldNotEndWithSlice,
}

pub struct SuffixRule<S, Mode, Kind> {
	suffix: S,
	mode: PhantomData<Mode>,
	kind: PhantomData<Kind>,
}

impl<M> SuffixRule<Unset, M, Unset> {
	#[must_use]
	pub fn new() -> Self {
		Self {
			suffix: Unset,
			mode: PhantomData,
			kind: PhantomData,
		}
	}

	#[must_use]
	pub fn str(self, suffix: &'static str) -> SuffixRule<&'static str, M, Str> {
		SuffixRule {
			suffix,
			mode: PhantomData,
			kind: PhantomData,
		}
	}

	pub fn slice<S>(self, suffix: S) -> SuffixRule<S, M, Slice> {
		SuffixRule {
			suffix,
			mode: PhantomData,
			kind: PhantomData,
		}
	}
}

impl<S, K> SuffixRule<S, Unset, K> {
	pub fn not(self) -> SuffixRule<S, Not, K> {
		SuffixRule {
			suffix: self.suffix,
			mode: PhantomData,
			kind: PhantomData,
		}
	}
}

impl<I: ?Sized, P, O> crate::Rule<I> for SuffixRule<P, Unset, Slice>
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
			Err(Error::ShouldEndWithSlice.into())
		}
	}
}

impl<I: ?Sized, P, O> crate::Rule<I> for SuffixRule<P, Not, Slice>
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
			Err(Error::ShouldNotEndWithSlice.into())
		} else {
			Ok(())
		}
	}
}

impl<I: ?Sized> crate::Rule<I> for SuffixRule<&'static str, Unset, Str>
where
	I: AsRef<str>,
{
	type Context = ();

	#[inline]
	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let inner = item.as_ref();
		let suffix = self.suffix;

		if inner.ends_with(suffix) {
			Ok(())
		} else {
			Err(Error::ShouldEndWith(self.suffix).into())
		}
	}
}

impl<I: ?Sized> crate::Rule<I> for SuffixRule<&'static str, Not, Str>
where
	I: AsRef<str>,
{
	type Context = ();

	#[inline]
	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let inner = item.as_ref();
		let suffix = self.suffix;

		if inner.ends_with(suffix) {
			Err(Error::ShouldNotEndWith(self.suffix).into())
		} else {
			Ok(())
		}
	}
}
