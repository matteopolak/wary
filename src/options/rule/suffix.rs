//! Rule for suffix validation.
//!
//! See [`SuffixRule`] for more information.

use core::fmt;

use crate::{options::{DebugDisplay, ItemSlice}, toolbox::rule::*};

#[doc(hidden)]
pub type Rule<S, Mode, Kind> = SuffixRule<S, Mode, Kind>;

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
	#[error("expected string to end with \"{0}\"")]
	ShouldEndWith(&'static str),
	#[error("expected string to not end with \"{0}\"")]
	ShouldNotEndWith(&'static str),
	#[error("expected slice to end with")]
	ShouldEndWithSlice(ItemSlice),
	#[error("expected slice to not end with")]
	ShouldNotEndWithSlice(ItemSlice),
}

pub struct Str;
pub struct Slice;
pub struct Not;

/// Rule for suffix validation.
///
/// # Example
///
/// ```
/// use wary::{Wary, Validate};
///
/// #[derive(Wary)]
/// struct Person {
///   #[validate(suffix(str = "hello"))]
///   name: String,
///   #[validate(suffix(slice = [5, 6, 7, 8]))]
///   numbers: Vec<u8>,
///   #[validate(suffix(not, str = "world"))]
///   greeting: String,
///   #[validate(suffix(not, slice = [1, 2, 3, 4]))]
///   more_numbers: Vec<u8>,
/// }
///
/// let person = Person {
///   name: "world hello".into(),
///   numbers: vec![1, 2, 3, 4, 5, 6, 7, 8],
///   greeting: "world hello".into(),
///   more_numbers: vec![5, 6, 7, 8, 9, 10],
/// };
///
/// assert!(person.validate(&()).is_ok());
///
/// let person = Person {
///   name: "hello world".into(),
///   numbers: vec![5, 6, 7, 8, 9, 10],
///   greeting: "hello world".into(),
///   more_numbers: vec![1, 2, 3, 4, 5, 6, 7, 8],
/// };
///
/// assert!(person.validate(&()).is_err());
/// ```
#[must_use]
pub struct SuffixRule<S, Mode, Kind> {
	suffix: S,
	mode: PhantomData<Mode>,
	kind: PhantomData<Kind>,
}

impl SuffixRule<Unset, Unset, Unset> {
	#[inline]
	pub const fn new() -> Self {
		Self {
			suffix: Unset,
			mode: PhantomData,
			kind: PhantomData,
		}
	}
}

impl<M> SuffixRule<Unset, M, Unset> {
	/// Ensure the input ends with the given string.
	#[inline]
	pub fn str(self, suffix: &'static str) -> SuffixRule<&'static str, M, Str> {
		SuffixRule {
			suffix,
			mode: PhantomData,
			kind: PhantomData,
		}
	}

	/// Ensure the input ends with the given slice.
	#[inline]
	pub fn slice<S>(self, suffix: S) -> SuffixRule<S, M, Slice> {
		SuffixRule {
			suffix,
			mode: PhantomData,
			kind: PhantomData,
		}
	}
}

impl<S, K> SuffixRule<S, Unset, K> {
	/// Inverts the rule.
	#[inline]
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
	P: AsSlice<Item = O> + fmt::Debug,
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
			Err(Error::ShouldEndWithSlice(DebugDisplay(&self.suffix).to_string()).into())
		}
	}
}

impl<I: ?Sized, P, O> crate::Rule<I> for SuffixRule<P, Not, Slice>
where
	I: AsSlice<Item = O>,
	P: AsSlice<Item = O> + fmt::Debug,
	O: PartialEq,
{
	type Context = ();

	#[inline]
	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let inner = item.as_slice();
		let suffix = self.suffix.as_slice();

		if inner.ends_with(suffix) {
			Err(Error::ShouldNotEndWithSlice(DebugDisplay(&self.suffix).to_string()).into())
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

#[cfg(test)]
mod test {
	use crate::toolbox::test::*;

	#[test]
	fn test_prefix_str_rule() {
		#[derive(Wary)]
		#[wary(crate = "crate")]
		struct Person<'name> {
			#[validate(suffix(str = "hello"))]
			name: Cow<'name, str>,
		}

		let person = Person {
			name: Cow::Borrowed("world hello"),
		};

		assert!(person.validate(&()).is_ok());

		let person = Person {
			name: Cow::Borrowed("hello world"),
		};

		assert!(person.validate(&()).is_err());
	}

	#[test]
	fn test_prefix_slice_rule() {
		#[derive(Wary)]
		#[wary(crate = "crate")]
		struct Person {
			#[validate(suffix(slice = [5, 6, 7, 8]))]
			name: Vec<u8>,
		}

		let person = Person {
			name: vec![1, 2, 3, 4, 5, 6, 7, 8],
		};

		assert!(person.validate(&()).is_ok());

		let person = Person {
			name: vec![5, 6, 7, 8, 9, 10],
		};

		assert!(person.validate(&()).is_err());
	}
}
