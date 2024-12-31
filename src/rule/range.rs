use std::{borrow::Cow, cmp::Ordering};

use crate::{Error, Validate};

#[doc(hidden)]
pub type Rule<'i, T, Min, Max> = RangeRule<'i, T, Min, Max>;

#[derive(Debug, thiserror::Error)]
pub enum RangeError {
	#[error("Value is too small")]
	TooSmall,
	#[error("Value is too large")]
	TooLarge,
}

pub trait Compare<B: ?Sized> {
	fn compare(&self, other: &B) -> Option<Ordering>;
}

pub struct RangeRule<'i, T, Min, Max> {
	min: Option<Min>,
	exclusive_min: bool,
	max: Option<Max>,
	exclusive_max: bool,
	inner: &'i T,
}

impl<'i, T, Min, Max> RangeRule<'i, T, Min, Max> {
	pub fn new(inner: &'i T) -> Self {
		Self {
			min: None,
			max: None,
			exclusive_min: false,
			exclusive_max: false,
			inner,
		}
	}

	pub fn min(mut self, min: Min) -> Self
	where
		T: Compare<Min>,
	{
		self.min = Some(min);
		self
	}

	pub fn max(mut self, max: Max) -> Self
	where
		T: Compare<Max>,
	{
		self.max = Some(max);
		self
	}

	pub fn exclusive_min(mut self, min: Min) -> Self
	where
		T: Compare<Min>,
	{
		self.min = Some(min);
		self.exclusive_min = true;
		self
	}

	pub fn exclusive_max(mut self, max: Max) -> Self
	where
		T: Compare<Max>,
	{
		self.max = Some(max);
		self.exclusive_max = true;
		self
	}
}

impl<T, Min, Max> Validate for RangeRule<'_, T, Min, Max>
where
	T: Compare<Min>,
	T: Compare<Max>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context) -> Result<(), Error> {
		if let Some(ref min) = self.min {
			match self.inner.compare(min) {
				Some(Ordering::Greater) => {}
				Some(Ordering::Equal) if !self.exclusive_min => {}
				_ => return Err(RangeError::TooSmall.into()),
			}
		}

		if let Some(ref max) = self.max {
			match self.inner.compare(max) {
				Some(Ordering::Less) => {}
				Some(Ordering::Equal) if !self.exclusive_max => {}
				_ => return Err(RangeError::TooLarge.into()),
			}
		}

		Ok(())
	}
}

impl<T> Compare<T> for T
where
	T: PartialOrd,
{
	fn compare(&self, other: &T) -> Option<Ordering> {
		self.partial_cmp(other)
	}
}

impl Compare<&str> for Cow<'_, str> {
	fn compare(&self, other: &&str) -> Option<Ordering> {
		self.as_ref().partial_cmp(*other)
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_integer_range() {
		for n in 1u32..=10 {
			let rule = RangeRule::new(&n).min(1).max(10);
			assert!(rule.validate(&()).is_ok());
		}

		let rule = RangeRule::new(&0u32).exclusive_min(0);
		assert!(rule.validate(&()).is_err());

		let rule = RangeRule::new(&11u32).exclusive_max(10);
		assert!(rule.validate(&()).is_err());

		let rule = RangeRule::new(&0u32).min(1);
		assert!(rule.validate(&()).is_err());

		let rule = RangeRule::new(&11u32).max(10);
		assert!(rule.validate(&()).is_err());
	}

	#[test]
	fn test_float_range() {
		for n in 1..=10 {
			let n = n as f32;

			let rule = RangeRule::new(&n).min(1.0).max(10.0);
			assert!(rule.validate(&()).is_ok());
		}

		let rule = RangeRule::new(&0.0f32).exclusive_min(0.0);
		assert!(rule.validate(&()).is_err());

		let rule = RangeRule::new(&11.0f32).exclusive_max(10.0);
		assert!(rule.validate(&()).is_err());

		let rule = RangeRule::new(&0.0f32).min(1.0);
		assert!(rule.validate(&()).is_err());

		let rule = RangeRule::new(&11.0f32).max(10.0);
		assert!(rule.validate(&()).is_err());
	}

	#[test]
	fn test_string_range() {
		let rule = RangeRule::new("hello").min("hello").max("world");
		assert!(rule.validate(&()).is_ok());

		let rule = RangeRule::new("hello").min("world");
		assert!(rule.validate(&()).is_err());

		let rule = RangeRule::new("world").max("hello");
		assert!(rule.validate(&()).is_err());

		let rule = RangeRule::new("hello").exclusive_min("hello");
		assert!(rule.validate(&()).is_err());

		let rule = RangeRule::new("world").exclusive_max("world");
		assert!(rule.validate(&()).is_err());

		let rule = RangeRule::new("world").min("worlds");
		assert!(rule.validate(&()).is_err());

		let rule = RangeRule::new("world").max("worl");
		assert!(rule.validate(&()).is_err());
	}
}
