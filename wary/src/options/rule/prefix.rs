//! Rule for prefix validation.
//!
//! See [`PrefixRule`] for more information.

use core::fmt;

use crate::{
	options::{DebugDisplay, ItemSlice},
	toolbox::rule::*,
};

#[doc(hidden)]
pub type Rule<P, Mode, Kind> = PrefixRule<P, Mode, Kind>;

pub struct Str;
pub struct Slice;

pub struct Not;

#[derive(Debug, thiserror::Error, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case", tag = "code"))]
pub enum Error {
	#[error("expected string to start with \"{value}\"")]
	ShouldStartWith { value: &'static str },
	#[error("expected string to not start with \"{value}\"")]
	ShouldNotStartWith { value: &'static str },
	#[error("expected slice to start with")]
	ShouldStartWithSlice { value: ItemSlice },
	#[error("expected slice to not start with")]
	ShouldNotStartWithSlice { value: ItemSlice },
}

/// Rule for prefix validation.
///
/// # Example
///
/// ```
/// use wary::{Wary, Validate};
///
/// #[derive(Wary)]
/// struct Person {
///   #[validate(prefix(str = "hello"))]
///   name: String,
///   #[validate(prefix(slice = [5, 6, 7, 8]))]
///   numbers: Vec<u8>,
///   #[validate(prefix(not, str = "hello"))]
///   greeting: String,
///   #[validate(prefix(not, slice = [1, 2, 3, 4]))]
///   more_numbers: Vec<u8>,
/// }
///
/// let person = Person {
///   name: "hello world".into(),
///   numbers: vec![5, 6, 7, 8, 9, 10],
///   greeting: "world hello".into(),
///   more_numbers: vec![5, 6, 7, 8, 9, 10],
/// };
///
/// assert!(person.validate(&()).is_ok());
///
/// let person = Person {
///   name: "world hello".into(),
///   numbers: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
///   greeting: "hello world".into(),
///   more_numbers: vec![1, 2, 3, 4, 5, 6, 7, 8],
/// };
///
/// assert!(person.validate(&()).is_err());
/// ```
#[must_use]
pub struct PrefixRule<P, Mode, Kind> {
	prefix: P,
	mode: PhantomData<Mode>,
	kind: PhantomData<Kind>,
}

impl PrefixRule<Unset, Unset, Unset> {
	#[inline]
	pub const fn new() -> Self {
		Self {
			prefix: Unset,
			mode: PhantomData,
			kind: PhantomData,
		}
	}
}

impl<M> PrefixRule<Unset, M, Unset> {
	/// Ensure the input starts with the given string.
	#[inline]
	pub fn str(self, prefix: &'static str) -> PrefixRule<&'static str, M, Str> {
		PrefixRule {
			prefix,
			mode: PhantomData,
			kind: PhantomData,
		}
	}

	/// Ensure the input starts with the given slice.
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
	/// Inverts the rule.
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
	P: AsSlice<Item = O> + fmt::Debug,
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
			Err(
				Error::ShouldStartWithSlice {
					value: DebugDisplay(&self.prefix).to_string(),
				}
				.into(),
			)
		}
	}
}

impl<I: ?Sized, P, O> crate::Rule<I> for PrefixRule<P, Not, Slice>
where
	I: AsSlice<Item = O>,
	P: AsSlice<Item = O> + fmt::Debug,
	O: PartialEq,
{
	type Context = ();

	#[inline]
	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let inner = item.as_slice();
		let prefix = self.prefix.as_slice();

		if inner.starts_with(prefix) {
			Err(
				Error::ShouldNotStartWithSlice {
					value: DebugDisplay(&self.prefix).to_string(),
				}
				.into(),
			)
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
			Err(Error::ShouldStartWith { value: self.prefix }.into())
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
			Err(Error::ShouldNotStartWith { value: self.prefix }.into())
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
