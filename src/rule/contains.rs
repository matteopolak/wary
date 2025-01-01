use core::fmt;
use std::marker::PhantomData;

use super::Unset;
use crate::{Error, Validate};

#[doc(hidden)]
pub type Rule<T, C, Mode> = ContainsRule<T, C, Mode>;

pub struct InOrder;
pub struct AnyOrder;

pub struct ContainsRule<T, C, Mode> {
	inner: T,
	contains: C,
	mode: PhantomData<Mode>,
}

impl<T> ContainsRule<T, Unset, InOrder> {
	pub fn new(inner: T) -> ContainsRule<T, Unset, InOrder> {
		ContainsRule {
			inner,
			contains: Unset,
			mode: PhantomData,
		}
	}
}

impl<T, M> ContainsRule<T, Unset, M> {
	pub fn item<C>(self, contains: C) -> ContainsRule<T, C, M> {
		ContainsRule {
			inner: self.inner,
			contains,
			mode: PhantomData,
		}
	}
}

impl<T, C, M> ContainsRule<T, C, M> {
	/// Validates that all of the items in the `contains` list are in the `inner`
	/// list in the same order.
	pub fn in_order(self) -> ContainsRule<T, C, InOrder> {
		ContainsRule {
			inner: self.inner,
			contains: self.contains,
			mode: PhantomData,
		}
	}

	/// Validates that all of the items in the `contains` list are in the `inner`
	/// list in any order. Note that this does not enforce the `inner` list to
	/// contain only the items in the `contains` list.
	pub fn any_order(self) -> ContainsRule<T, C, AnyOrder> {
		ContainsRule {
			inner: self.inner,
			contains: self.contains,
			mode: PhantomData,
		}
	}
}

struct SliceDisplay<T>(Box<[T]>);

impl<T> SliceDisplay<T> {
	fn from_slice(slice: &[T]) -> Self
	where
		T: Clone,
	{
		SliceDisplay(slice.to_vec().into_boxed_slice())
	}

	fn boxed(self) -> Box<dyn fmt::Display>
	where
		T: fmt::Display + 'static,
	{
		Box::new(self)
	}
}

impl<T> fmt::Display for SliceDisplay<T>
where
	T: fmt::Display,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut iter = self.0.iter();

		if let Some(first) = iter.next() {
			write!(f, "{}", first)?;

			for item in iter {
				write!(f, ", {}", item)?;
			}
		}

		Ok(())
	}
}

impl<T, C, I> Validate for ContainsRule<T, C, InOrder>
where
	T: ToSlice<Item = I>,
	C: ToSlice<Item = I>,
	I: PartialEq + fmt::Display + Clone + 'static,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context) -> Result<(), Error> {
		let inner = self.inner.to_slice();
		let contains = self.contains.to_slice();

		let Some(first) = contains.first() else {
			return Ok(());
		};

		let mut inner_iter = inner.iter();

		while let Some(inner_item) = inner_iter.next() {
			if inner_item == first && inner_iter.as_slice().starts_with(contains) {
				return Ok(());
			}
		}

		// TODO: this is terrible
		Err(Error::Contains(SliceDisplay::from_slice(contains).boxed()))
	}
}

impl<T, C, I> Validate for ContainsRule<T, C, AnyOrder>
where
	T: ToSlice<Item = I>,
	C: ToSlice<Item = I>,
	I: PartialEq + fmt::Display + Clone + 'static,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context) -> Result<(), Error> {
		let inner = self.inner.to_slice();
		let contains = self.contains.to_slice();

		for item in contains {
			if !inner.contains(item) {
				return Err(Error::Contains(Box::new(item.clone())));
			}
		}

		Ok(())
	}
}

pub trait ToSlice {
	type Item;

	fn to_slice(&self) -> &[Self::Item];
}

impl<T> ToSlice for &T
where
	T: ToSlice,
{
	type Item = T::Item;

	fn to_slice(&self) -> &[Self::Item] {
		(*self).to_slice()
	}
}

impl ToSlice for &str {
	type Item = u8;

	fn to_slice(&self) -> &[Self::Item] {
		self.as_bytes()
	}
}

impl<T> ToSlice for Vec<T> {
	type Item = T;

	fn to_slice(&self) -> &[Self::Item] {
		self
	}
}

impl<T> ToSlice for [T] {
	type Item = T;

	fn to_slice(&self) -> &[Self::Item] {
		self
	}
}

impl<const N: usize, T> ToSlice for [T; N] {
	type Item = T;

	fn to_slice(&self) -> &[Self::Item] {
		self
	}
}

impl ToSlice for str {
	type Item = u8;

	fn to_slice(&self) -> &[Self::Item] {
		self.as_bytes()
	}
}

impl ToSlice for String {
	type Item = u8;

	fn to_slice(&self) -> &[Self::Item] {
		self.as_bytes()
	}
}
