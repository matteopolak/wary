use core::fmt;

use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule<C, Mode, Kind> = ContainsRule<C, Mode, Kind>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("expected string to contain \"{0}\"")]
	ShouldContain(&'static str),
	#[error("found unexpected string \"{item}\" at position {position}")]
	ShouldNotContain { position: usize, item: &'static str },
	#[error("expected slice to contain")]
	ShouldContainSlice,
	#[error("found unexpected item at position {position}")]
	ShouldNotContainSlice { position: usize },
}

pub struct InOrder;
pub struct AnyOrder;
pub struct InOrderNot;
pub struct AnyOrderNot;

pub struct Str;
pub struct Slice;

pub struct ContainsRule<C, Mode, Kind> {
	contains: C,
	mode: PhantomData<Mode>,
	kind: PhantomData<Kind>,
}

impl ContainsRule<Unset, InOrder, Unset> {
	#[must_use]
	pub fn new() -> ContainsRule<Unset, InOrder, Unset> {
		ContainsRule {
			contains: Unset,
			mode: PhantomData,
			kind: PhantomData,
		}
	}
}

impl<M> ContainsRule<Unset, M, Unset> {
	#[must_use]
	pub fn str(self, contains: &'static str) -> ContainsRule<&'static str, M, Str> {
		ContainsRule {
			contains,
			mode: PhantomData,
			kind: PhantomData,
		}
	}

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
	/// list in the same order.
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
	pub fn any_order(self) -> ContainsRule<C, AnyOrder, K> {
		ContainsRule {
			contains: self.contains,
			mode: PhantomData,
			kind: PhantomData,
		}
	}
}

impl<C, K> ContainsRule<C, InOrder, K> {
	/// Validates that all of the items in the `contains` list are not in the
	/// `inner` list in the same order.
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
	C: AsSlice<Item = O>,
	O: PartialEq + fmt::Display + Clone + 'static,
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

		Err(Error::ShouldContainSlice.into())
	}
}

impl<I, C, D> crate::Rule<I> for ContainsRule<C, InOrderNot, Slice>
where
	I: AsSlice<Item = D>,
	C: AsSlice<Item = D>,
	D: PartialEq + fmt::Display + Clone + 'static,
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
				return Err(Error::ShouldNotContainSlice { position: idx }.into());
			}

			idx += 1;
		}

		Ok(())
	}
}

impl<I, C, O> crate::Rule<I> for ContainsRule<C, AnyOrder, Slice>
where
	I: AsSlice<Item = O>,
	C: AsSlice<Item = O> + fmt::Display,
	O: PartialEq + fmt::Display + Clone + 'static,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let inner = item.as_slice();
		let contains = self.contains.as_slice();

		for item in contains {
			if !inner.contains(item) {
				return Err(Error::ShouldContainSlice.into());
			}
		}

		Ok(())
	}
}

impl<I, C, D> crate::Rule<I> for ContainsRule<C, AnyOrderNot, Slice>
where
	I: AsSlice<Item = D>,
	C: AsSlice<Item = D>,
	D: PartialEq + fmt::Display + Clone + 'static,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let inner = item.as_slice();
		let contains = self.contains.as_slice();

		for (idx, item) in contains.iter().enumerate() {
			if inner.contains(item) {
				return Err(Error::ShouldNotContainSlice { position: idx }.into());
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
			Err(Error::ShouldContain(self.contains).into())
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
					item: self.contains,
				}
				.into(),
			)
		} else {
			Ok(())
		}
	}
}
