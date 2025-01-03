use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule<P, Mode, Kind> = PrefixRule<P, Mode, Kind>;

pub struct Str;
pub struct Slice;

pub struct Not;

#[derive(Debug, thiserror::Error, PartialEq)]
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

#[must_use]
pub struct PrefixRule<P, Mode, Kind> {
	prefix: P,
	mode: PhantomData<Mode>,
	kind: PhantomData<Kind>,
}

impl PrefixRule<Unset, Unset, Unset> {
	#[inline]
	pub fn new() -> Self {
		Self {
			prefix: Unset,
			mode: PhantomData,
			kind: PhantomData,
		}
	}
}

impl<M> PrefixRule<Unset, M, Unset> {
	#[inline]
	pub fn str(self, prefix: &'static str) -> PrefixRule<&'static str, M, Str> {
		PrefixRule {
			prefix,
			mode: PhantomData,
			kind: PhantomData,
		}
	}

	#[inline]
	pub fn slice<P>(self, prefix: P) -> PrefixRule<P, M, Slice> {
		PrefixRule {
			prefix,
			mode: PhantomData,
			kind: PhantomData,
		}
	}
}

impl<P, K> PrefixRule<P, Unset, K> {
	#[inline]
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

#[cfg(test)]
mod test {
	use std::borrow::Cow;

	use crate::toolbox::test::*;

	#[test]
	fn test_prefix_str_rule() {
		#[derive(Wary)]
		#[wary(crate = "crate")]
		struct Person<'name> {
			#[validate(prefix(str = "hello"))]
			name: Cow<'name, str>,
		}

		let person = Person {
			name: Cow::Borrowed("hello world"),
		};

		assert!(person.validate(&()).is_ok());

		let person = Person {
			name: Cow::Borrowed("world hello"),
		};

		assert!(person.validate(&()).is_err());
	}

	#[test]
	fn test_prefix_slice_rule() {
		#[derive(Wary)]
		#[wary(crate = "crate")]
		struct Person {
			#[validate(prefix(slice = [5, 6, 7, 8]))]
			name: Vec<u8>,
		}

		let person = Person {
			name: vec![5, 6, 7, 8, 9, 10],
		};

		assert!(person.validate(&()).is_ok());

		let person = Person {
			name: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
		};

		assert!(person.validate(&()).is_err());
	}
}
