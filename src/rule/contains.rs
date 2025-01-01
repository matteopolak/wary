use core::fmt;

use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule_<C, Mode> = ContainsRule<C, Mode>;

pub struct InOrder;
pub struct AnyOrder;

pub struct ContainsRule<C, Mode> {
	contains: C,
	mode: PhantomData<Mode>,
}

impl ContainsRule<Unset, InOrder> {
	pub fn new() -> ContainsRule<Unset, InOrder> {
		ContainsRule {
			contains: Unset,
			mode: PhantomData,
		}
	}
}

impl<M> ContainsRule<Unset, M> {
	pub fn item<C>(self, contains: C) -> ContainsRule<C, M> {
		ContainsRule {
			contains,
			mode: PhantomData,
		}
	}
}

impl<C, M> ContainsRule<C, M> {
	/// Validates that all of the items in the `contains` list are in the `inner`
	/// list in the same order.
	pub fn in_order(self) -> ContainsRule<C, InOrder> {
		ContainsRule {
			contains: self.contains,
			mode: PhantomData,
		}
	}

	/// Validates that all of the items in the `contains` list are in the `inner`
	/// list in any order. Note that this does not enforce the `inner` list to
	/// contain only the items in the `contains` list.
	pub fn any_order(self) -> ContainsRule<C, AnyOrder> {
		ContainsRule {
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

impl<I, C, O> Rule<I> for ContainsRule<C, InOrder>
where
	I: AsSlice<Item = O>,
	C: AsSlice<Item = O>,
	O: PartialEq + fmt::Display + Clone + 'static,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<(), Error> {
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

		// TODO: this is terrible
		Err(Error::Contains(SliceDisplay::from_slice(contains).boxed()))
	}
}

impl<I, C, O> Rule<I> for ContainsRule<C, AnyOrder>
where
	I: AsSlice<Item = O>,
	C: AsSlice<Item = O>,
	O: PartialEq + fmt::Display + Clone + 'static,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<(), Error> {
		let inner = item.as_slice();
		let contains = self.contains.as_slice();

		for item in contains {
			if !inner.contains(item) {
				return Err(Error::Contains(Box::new(item.clone())));
			}
		}

		Ok(())
	}
}
