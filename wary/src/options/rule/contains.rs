//! Rule for validation of slice or string containments.
//!
//! See [`ContainsRule`] for more information.

use core::fmt;

use crate::{
	options::{DebugDisplay, ItemSlice},
	toolbox::rule::*,
};

#[doc(hidden)]
pub type Rule<C, Mode, Kind> = ContainsRule<C, Mode, Kind>;

#[derive(Debug, thiserror::Error, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case", tag = "code"))]
pub enum Error {
	#[error("expected string to contain \"{value}\"")]
	ShouldContain { value: &'static str },
	#[error("found unexpected string \"{value}\" at position {position}")]
	ShouldNotContain {
		position: usize,
		value: &'static str,
	},
	#[error("expected slice to contain")]
	ShouldContainSlice { value: ItemSlice },
	#[error("found unexpected value at position {position}")]
	ShouldNotContainSlice { position: usize, value: ItemSlice },
}

impl Error {
	#[must_use]
	pub fn code(&self) -> &'static str {
		match self {
			Self::ShouldContain { .. } => "should_contain",
			Self::ShouldNotContain { .. } => "should_not_contain",
			Self::ShouldContainSlice { .. } => "should_contain_slice",
			Self::ShouldNotContainSlice { .. } => "should_not_contain_slice",
		}
	}

	#[cfg(feature = "alloc")]
	#[must_use]
	pub fn message(&self) -> Cow<'static, str> {
		match self {
			Self::ShouldContain { value } => format!("expected to contain {value}"),
			Self::ShouldNotContain { position, value } => {
				format!("found unexpected value at position {position}: {value}")
			}
			Self::ShouldContainSlice { value } => format!("expected to contain {value:?}"),
			Self::ShouldNotContainSlice { position, value } => {
				format!("found unexpected value at position {position}: {value:?}")
			}
		}
		.into()
	}

	#[cfg(not(feature = "alloc"))]
	pub fn message(&self) -> &'static str {
		match self {
			Self::ShouldContain { .. } => "expected to contain",
			Self::ShouldNotContain { .. } => "found unexpected value",
			Self::ShouldContainSlice { .. } => "expected to contain",
			Self::ShouldNotContainSlice { .. } => "found unexpected value",
		}
	}
}

pub struct InOrder;
pub struct AnyOrder;
pub struct InOrderNot;
pub struct AnyOrderNot;

pub struct Str;
pub struct Slice;

/// Rule for validation of slice or string containments.
///
/// # Example
///
/// ```
/// use wary::{Wary, Validate};
///
/// #[derive(Wary)]
/// struct Person {
///   #[validate(contains(str = "hello"))]
///   name: String,
///   #[validate(contains(slice = [5, 6, 7, 8]))]
///   numbers: Vec<u8>,
///   #[validate(contains(any_order, slice = [5, 6, 7, 8]))]
///   greeting: Vec<u8>,
/// }
///
/// let person = Person {
///   name: "abchelloxyz".into(),
///   numbers: vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
///   greeting: vec![8, 6, 7, 5],
/// };
///
/// assert!(person.validate(&()).is_ok());
///
/// let person = Person {
///   name: "abcworldxyz".into(),
///   numbers: vec![1, 2, 3, 4, 5, 6, 7, 9],
///   greeting: vec![3, 4, 5, 6],
/// };
///
/// assert!(person.validate(&()).is_err());
/// ```
#[must_use]
pub struct ContainsRule<C, Mode, Kind> {
	contains: C,
	mode: PhantomData<Mode>,
	kind: PhantomData<Kind>,
}

impl ContainsRule<Unset, InOrder, Unset> {
	#[inline]
	pub const fn new() -> ContainsRule<Unset, InOrder, Unset> {
		ContainsRule {
			contains: Unset,
			mode: PhantomData,
			kind: PhantomData,
		}
	}
}

impl<M> ContainsRule<Unset, M, Unset> {
	/// Ensure the input contains the given string.
	#[inline]
	pub fn str(self, contains: &'static str) -> ContainsRule<&'static str, M, Str> {
		ContainsRule {
			contains,
			mode: PhantomData,
			kind: PhantomData,
		}
	}

	/// Ensure the input contains the given slice.
	#[inline]
	pub fn slice<C>(self, contains: C) -> ContainsRule<C, M, Slice> {
		ContainsRule {
			contains,
			mode: PhantomData,
			kind: PhantomData,
		}
	}
}

impl<C, M, K> ContainsRule<C, M, K> {
	/// Validates that all of the items in the `contains` list are in the `inner`
	/// list in the same order. This is the default behavior.
	#[inline]
	pub fn in_order(self) -> ContainsRule<C, InOrder, K> {
		ContainsRule {
			contains: self.contains,
			mode: PhantomData,
			kind: PhantomData,
		}
	}

	/// Validates that all of the items in the `contains` list are in the `inner`
	/// list in any order. Note that this does not enforce the `inner` list to
	/// contain only the items in the `contains` list.
	///
	/// This can only be used with slices.
	#[inline]
	pub fn any_order(self) -> ContainsRule<C, AnyOrder, K> {
		ContainsRule {
			contains: self.contains,
			mode: PhantomData,
			kind: PhantomData,
		}
	}
}

impl<C, K> ContainsRule<C, InOrder, K> {
	/// Inverts the rule.
	#[inline]
	pub fn not(self) -> ContainsRule<C, InOrderNot, K> {
		ContainsRule {
			contains: self.contains,
			mode: PhantomData,
			kind: PhantomData,
		}
	}
}

impl<C, K> ContainsRule<C, AnyOrder, K> {
	/// Validates that all of the items in the `contains` list are not in the
	/// `inner` list in any order. Note that this does not enforce the `inner`
	/// list to contain only the items in the `contains` list.
	#[inline]
	pub fn not(self) -> ContainsRule<C, AnyOrderNot, K> {
		ContainsRule {
			contains: self.contains,
			mode: PhantomData,
			kind: PhantomData,
		}
	}
}

impl<I, C, O> crate::Rule<I> for ContainsRule<C, InOrder, Slice>
where
	I: AsSlice<Item = O>,
	C: AsSlice<Item = O> + fmt::Debug,
	O: PartialEq,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let inner = item.as_slice();
		let contains = self.contains.as_slice();

		let [first, contains @ ..] = contains else {
			return Ok(());
		};

		let mut inner_iter = inner.iter();

		while let Some(inner_item) = inner_iter.next() {
			if inner_item == first && inner_iter.as_slice().starts_with(contains) {
				return Ok(());
			}
		}

		Err(
			Error::ShouldContainSlice {
				value: DebugDisplay(&self.contains).to_string(),
			}
			.into(),
		)
	}
}

impl<I, C, D> crate::Rule<I> for ContainsRule<C, InOrderNot, Slice>
where
	I: AsSlice<Item = D>,
	C: AsSlice<Item = D> + fmt::Debug,
	D: PartialEq,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let inner = item.as_slice();
		let contains = self.contains.as_slice();

		let [first, contains @ ..] = contains else {
			return Ok(());
		};

		let mut inner_iter = inner.iter();
		let mut idx = 0;

		while let Some(inner_item) = inner_iter.next() {
			if inner_item == first && inner_iter.as_slice().starts_with(contains) {
				return Err(
					Error::ShouldNotContainSlice {
						position: idx,
						value: DebugDisplay(&self.contains).to_string(),
					}
					.into(),
				);
			}

			idx += 1;
		}

		Ok(())
	}
}

impl<I, C, O> crate::Rule<I> for ContainsRule<C, AnyOrder, Slice>
where
	I: AsSlice<Item = O>,
	C: AsSlice<Item = O> + fmt::Debug,
	O: PartialEq,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let inner = item.as_slice();
		let contains = self.contains.as_slice();

		for item in contains {
			if !inner.contains(item) {
				return Err(
					Error::ShouldContainSlice {
						value: DebugDisplay(&self.contains).to_string(),
					}
					.into(),
				);
			}
		}

		Ok(())
	}
}

impl<I, C, D> crate::Rule<I> for ContainsRule<C, AnyOrderNot, Slice>
where
	I: AsSlice<Item = D>,
	C: AsSlice<Item = D> + fmt::Debug,
	D: PartialEq,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let inner = item.as_slice();
		let contains = self.contains.as_slice();

		for (idx, item) in contains.iter().enumerate() {
			if inner.contains(item) {
				return Err(
					Error::ShouldNotContainSlice {
						position: idx,
						value: DebugDisplay(&self.contains).to_string(),
					}
					.into(),
				);
			}
		}

		Ok(())
	}
}

impl<I> crate::Rule<I> for ContainsRule<&'static str, InOrder, Str>
where
	I: AsRef<str>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let inner = item.as_ref();
		let contains = self.contains;

		if inner.contains(contains) {
			Ok(())
		} else {
			Err(
				Error::ShouldContain {
					value: self.contains,
				}
				.into(),
			)
		}
	}
}

impl<I> crate::Rule<I> for ContainsRule<&'static str, InOrderNot, Str>
where
	I: AsRef<str>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let inner = item.as_ref();
		let contains = self.contains;

		if let Some(idx) = inner.find(contains) {
			Err(
				Error::ShouldNotContain {
					position: idx,
					value: self.contains,
				}
				.into(),
			)
		} else {
			Ok(())
		}
	}
}

#[cfg(test)]
mod test {
	use crate::toolbox::test::*;

	#[test]
	fn test_contains_str_rule() {
		#[derive(Wary)]
		#[wary(crate = "crate")]
		struct Person<'name> {
			#[validate(contains(str = "hello"))]
			name: Cow<'name, str>,
		}

		let person = Person {
			name: Cow::Borrowed("abchelloxyz"),
		};

		assert!(person.validate(&()).is_ok());

		let person = Person {
			name: Cow::Borrowed("abcworldxyz"),
		};

		assert!(person.validate(&()).is_err());
	}

	#[test]
	fn test_contains_slice_rule() {
		#[derive(Wary)]
		#[wary(crate = "crate")]
		struct Person {
			#[validate(contains(slice = [5, 6, 7, 8]))]
			name: Vec<u8>,
		}

		let person = Person {
			name: vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
		};

		assert!(person.validate(&()).is_ok());

		let person = Person {
			name: vec![1, 2, 3, 4, 5, 6, 7, 9],
		};

		assert!(person.validate(&()).is_err());
	}
}
