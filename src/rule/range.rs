use std::{borrow::Cow, cmp::Ordering};

use crate::{Error, Validate};

#[doc(hidden)]
pub type Rule<T> = RangeRule<T>;

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum RangeError {
	#[error("Value is too small")]
	TooSmall,
	#[error("Value is too large")]
	TooLarge,
}

pub trait Compare<B: ?Sized> {
	fn compare(&self, other: &B) -> Option<Ordering>;
}

pub struct RangeRule<T> {
	error: Option<RangeError>,
	inner: T,
}

impl<T> RangeRule<T> {
	pub fn new(inner: T) -> Self {
		Self { error: None, inner }
	}

	pub fn min<Min>(mut self, min: Min) -> Self
	where
		T: Compare<Min>,
	{
		if let Some(Ordering::Less) = self.inner.compare(&min) {
			self.error = Some(RangeError::TooSmall);
		}
		self
	}

	pub fn max<Max>(mut self, max: Max) -> Self
	where
		T: Compare<Max>,
	{
		if let Some(Ordering::Greater) = self.inner.compare(&max) {
			self.error = Some(RangeError::TooLarge);
		}
		self
	}

	pub fn exclusive_min<Min>(mut self, min: Min) -> Self
	where
		T: Compare<Min>,
	{
		if let Some(Ordering::Equal | Ordering::Less) = self.inner.compare(&min) {
			self.error = Some(RangeError::TooSmall);
		}
		self
	}

	pub fn exclusive_max<Max>(mut self, max: Max) -> Self
	where
		T: Compare<Max>,
	{
		if let Some(Ordering::Equal | Ordering::Greater) = self.inner.compare(&max) {
			self.error = Some(RangeError::TooLarge);
		}
		self
	}
}

impl<T> Validate for RangeRule<T>
where
	for<'a> &'a T: Compare<&'a T>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context) -> Result<(), Error> {
		if let Some(error) = self.error {
			return Err(error.into());
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

impl Compare<Cow<'_, str>> for str {
	fn compare(&self, other: &Cow<'_, str>) -> Option<Ordering> {
		self.partial_cmp(other.as_ref())
	}
}

impl Compare<&&str> for &'_ Cow<'_, str> {
	fn compare(&self, other: &&&str) -> Option<Ordering> {
		self.as_ref().partial_cmp(other)
	}
}

impl Compare<&&str> for &'_ String {
	fn compare(&self, other: &&&str) -> Option<Ordering> {
		self.as_str().partial_cmp(other)
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_integer_range() {
		for n in 1u32..=10 {
			let rule = RangeRule::new(&n).min(&1).max(&10);
			assert!(rule.validate(&()).is_ok());
		}

		let rule = RangeRule::new(&0u32).exclusive_min(&0);
		assert!(rule.validate(&()).is_err());

		let rule = RangeRule::new(&11u32).exclusive_max(&10);
		assert!(rule.validate(&()).is_err());

		let rule = RangeRule::new(&0u32).min(&1);
		assert!(rule.validate(&()).is_err());

		let rule = RangeRule::new(&11u32).max(&10);
		assert!(rule.validate(&()).is_err());
	}

	#[test]
	fn test_float_range() {
		for n in 1..=10 {
			let n = n as f32;

			let rule = RangeRule::new(&n).min(&1.0).max(&10.0);
			assert!(rule.validate(&()).is_ok());
		}

		let rule = RangeRule::new(&0.0f32).exclusive_min(&0.0);
		assert!(rule.validate(&()).is_err());

		let rule = RangeRule::new(&11.0f32).exclusive_max(&10.0);
		assert!(rule.validate(&()).is_err());

		let rule = RangeRule::new(&0.0f32).min(&1.0);
		assert!(rule.validate(&()).is_err());

		let rule = RangeRule::new(&11.0f32).max(&10.0);
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
