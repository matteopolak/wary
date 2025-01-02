#![allow(clippy::should_implement_trait)]

use core::fmt;

use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule<C, Mode> = Contains<C, Mode>;

// TODO: somehow represent the slice?
type Item = ();

#[derive(thiserror::Error)]
pub enum Error {
	#[error("expected to contain")]
	Contains(Item),
	#[error("expected to not contain")]
	NotContains {
		position: usize,
		item: Item
	}
}

struct DebugDisplay<T>(T);

impl<T> fmt::Debug for DebugDisplay<T>
where
	T: fmt::Display,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl fmt::Debug for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	  match self {
			Self::Contains(item) => f.debug_tuple("Contains").field(item).finish(),
			Self::NotContains { position, item } => f.debug_struct("NotContains")
				.field("position", position)
				.field("item", item)
				.finish(),
		}
	}
}

pub struct InOrder;
pub struct AnyOrder;
pub struct InOrderNot;
pub struct AnyOrderNot;

pub struct Contains<C, Mode> {
	contains: C,
	mode: PhantomData<Mode>,
}

impl Contains<Unset, InOrder> {
	pub fn new() -> Contains<Unset, InOrder> {
		Contains {
			contains: Unset,
			mode: PhantomData,
		}
	}
}

impl<M> Contains<Unset, M> {
	pub fn item<C>(self, contains: C) -> Contains<C, M> {
		Contains {
			contains,
			mode: PhantomData,
		}
	}
}

impl<C, M> Contains<C, M> {
	/// Validates that all of the items in the `contains` list are in the `inner`
	/// list in the same order.
	pub fn in_order(self) -> Contains<C, InOrder> {
		Contains {
			contains: self.contains,
			mode: PhantomData,
		}
	}

	/// Validates that all of the items in the `contains` list are in the `inner`
	/// list in any order. Note that this does not enforce the `inner` list to
	/// contain only the items in the `contains` list.
	pub fn any_order(self) -> Contains<C, AnyOrder> {
		Contains {
			contains: self.contains,
			mode: PhantomData,
		}
	}
}

impl<C> Contains<C, InOrder> {
	/// Validates that all of the items in the `contains` list are not in the `inner`
	/// list in the same order.
	pub fn not(self) -> Contains<C, InOrderNot> {
		Contains {
			contains: self.contains,
			mode: PhantomData,
		}
	}
}

impl<C> Contains<C, AnyOrder> {
	/// Validates that all of the items in the `contains` list are not in the `inner`
	/// list in any order. Note that this does not enforce the `inner` list to
	/// contain only the items in the `contains` list.
	pub fn not(self) -> Contains<C, AnyOrderNot> {
		Contains {
			contains: self.contains,
			mode: PhantomData,
		}
	}
}

impl<I, C, O> crate::Rule<I> for Contains<C, InOrder>
where
	I: AsSlice<Item = O>,
	C: AsSlice<Item = O>,
	O: PartialEq + fmt::Display + Clone + 'static,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let inner = item.as_slice();
		let contains = self.contains.as_slice();

		let Some(first) = contains.first() else {
			return Ok(());
		};

		let mut inner_iter = inner.iter();

		while let Some(inner_item) = inner_iter.next() {
			if inner_item == first && inner_iter.as_slice().starts_with(contains) {
				return Ok(());
			}
		}

		Err(Error::Contains(()).into())
	}
}

impl<I, C, D> crate::Rule<I> for Contains<C, InOrderNot>
where
	I: AsSlice<Item = D>,
	C: AsSlice<Item = D>,
	D: PartialEq + fmt::Display + Clone + 'static,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let inner = item.as_slice();
		let contains = self.contains.as_slice();

		let Some(first) = contains.first() else {
			return Ok(());
		};

		let mut inner_iter = inner.iter();
		let mut idx = 0;

		while let Some(inner_item) = inner_iter.next() {
			if inner_item == first && inner_iter.as_slice().starts_with(contains) {
				return Err(Error::NotContains {
					item: (),
					position: idx,
				}.into());
			}

			idx += 1;
		}

		Ok(())
	}
}

impl<I, C, O> crate::Rule<I> for Contains<C, AnyOrder>
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
				return Err(Error::Contains(()).into());
			}
		}

		Ok(())
	}
}

impl<I, C, D> crate::Rule<I> for Contains<C, AnyOrderNot>
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
				return Err(Error::NotContains {
					item: (),
					position: idx,
				}.into());
			}
		}

		Ok(())
	}
}
