use std::{borrow::Cow, cmp::Ordering};

use crate::{Error, Validate};

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

pub struct RangeRule<T, B> {
	min: Option<B>,
	exclusive_min: bool,
	max: Option<B>,
	exclusive_max: bool,
	inner: T,
}

impl<T, B> RangeRule<T, B> {
	pub fn new(inner: T) -> Self {
		Self {
			min: None,
			max: None,
			exclusive_min: false,
			exclusive_max: false,
			inner,
		}
	}

	pub fn min(mut self, min: B) -> Self
	where
		T: Compare<B>,
	{
		self.min = Some(min);
		self
	}

	pub fn max(mut self, max: B) -> Self
	where
		T: Compare<B>,
	{
		self.max = Some(max);
		self
	}

	pub fn exclusive_min(mut self, min: B) -> Self
	where
		T: Compare<B>,
	{
		self.min = Some(min);
		self.exclusive_min = true;
		self
	}

	pub fn exclusive_max(mut self, max: B) -> Self
	where
		T: Compare<B>,
	{
		self.max = Some(max);
		self.exclusive_max = true;
		self
	}
}

impl<T, B> Validate for RangeRule<T, B>
where
	T: Compare<B>,
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

macro_rules! impl_compare {
	($($t:ty),*) => {
		$(
			impl Compare<$t> for $t {
				fn compare(&self, other: &$t) -> Option<Ordering> {
					self.partial_cmp(other)
				}
			}
		)*
	};
}

impl_compare!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64);

impl<T> Compare<&T> for &T
where
	T: Compare<T> + ?Sized,
{
	fn compare(&self, other: &&T) -> Option<Ordering> {
		(*self).compare(other)
	}
}

impl<T> Compare<T> for Option<T>
where
	T: Compare<T>,
{
	fn compare(&self, other: &T) -> Option<Ordering> {
		match self {
			Some(inner) => inner.compare(other),
			None => None,
		}
	}
}

impl<T> Compare<&T> for &Option<T>
where
	T: Compare<T>,
{
	fn compare(&self, other: &&T) -> Option<Ordering> {
		match self {
			Some(inner) => inner.compare(other),
			None => None,
		}
	}
}

impl Compare<String> for String {
	fn compare(&self, other: &String) -> Option<Ordering> {
		self.as_str().compare(other.as_str())
	}
}

impl Compare<str> for str {
	fn compare(&self, other: &str) -> Option<Ordering> {
		self.partial_cmp(other)
	}
}

impl Compare<String> for str {
	fn compare(&self, other: &String) -> Option<Ordering> {
		self.compare(other.as_str())
	}
}

impl Compare<str> for String {
	fn compare(&self, other: &str) -> Option<Ordering> {
		self.as_str().compare(other)
	}
}

impl<T> Compare<Cow<'_, T>> for Cow<'_, T>
where
	T: ?Sized + PartialOrd + ToOwned,
{
	fn compare(&self, other: &Cow<'_, T>) -> Option<Ordering> {
		self.as_ref().partial_cmp(other.as_ref())
	}
}

impl<T> Compare<T> for Cow<'_, T>
where
	T: ?Sized + Compare<T> + ToOwned,
{
	fn compare(&self, other: &T) -> Option<Ordering> {
		self.as_ref().compare(other)
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_integer_range() {
		for n in 1u32..=10 {
			let rule = RangeRule::new(n).min(1).max(10);
			assert!(rule.validate(&()).is_ok());
		}

		let rule = RangeRule::new(0u32).exclusive_min(0);
		assert!(rule.validate(&()).is_err());

		let rule = RangeRule::new(11u32).exclusive_max(10);
		assert!(rule.validate(&()).is_err());

		let rule = RangeRule::new(0u32).min(1);
		assert!(rule.validate(&()).is_err());

		let rule = RangeRule::new(11u32).max(10);
		assert!(rule.validate(&()).is_err());
	}

	#[test]
	fn test_float_range() {
		for n in 1..=10 {
			let n = n as f32;

			let rule = RangeRule::new(n).min(1.0).max(10.0);
			assert!(rule.validate(&()).is_ok());
		}

		let rule = RangeRule::new(0.0f32).exclusive_min(0.0);
		assert!(rule.validate(&()).is_err());

		let rule = RangeRule::new(11.0f32).exclusive_max(10.0);
		assert!(rule.validate(&()).is_err());

		let rule = RangeRule::new(0.0f32).min(1.0);
		assert!(rule.validate(&()).is_err());

		let rule = RangeRule::new(11.0f32).max(10.0);
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
