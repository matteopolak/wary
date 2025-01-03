use std::{borrow::Cow, cmp::Ordering};

use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule<Min, Max> = RangeRule<Min, Max>;

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
	#[error("Value is too small")]
	TooSmall,
	#[error("Value is too large")]
	TooLarge,
}

pub trait Compare<B: ?Sized = Self> {
	fn compare(&self, other: &B) -> Option<Ordering>;
}

pub struct RangeRule<Min, Max> {
	min: Option<Min>,
	max: Option<Max>,
	exclusive_min: bool,
	exclusive_max: bool,
}

impl RangeRule<Unset, Unset> {
	#[must_use]
	pub fn new() -> Self {
		RangeRule {
			min: None,
			max: None,
			exclusive_min: false,
			exclusive_max: false,
		}
	}
}

impl<Max> RangeRule<Unset, Max> {
	pub fn min<Min>(self, min: Min) -> RangeRule<Min, Max> {
		RangeRule {
			min: Some(min),
			max: self.max,
			exclusive_min: false,
			exclusive_max: self.exclusive_max,
		}
	}

	pub fn exclusive_min<Min>(self, min: Min) -> RangeRule<Min, Max> {
		RangeRule {
			min: Some(min),
			max: self.max,
			exclusive_min: true,
			exclusive_max: self.exclusive_max,
		}
	}
}

impl<Min> RangeRule<Min, Unset> {
	pub fn max<Max>(self, max: Max) -> RangeRule<Min, Max> {
		RangeRule {
			min: self.min,
			max: Some(max),
			exclusive_min: self.exclusive_min,
			exclusive_max: false,
		}
	}

	pub fn exclusive_max<Max>(self, max: Max) -> RangeRule<Min, Max> {
		RangeRule {
			min: self.min,
			max: Some(max),
			exclusive_min: self.exclusive_min,
			exclusive_max: true,
		}
	}
}

impl<I: ?Sized, Min, Max> crate::Rule<I> for RangeRule<Min, Max>
where
	I: Compare<Min> + Compare<Max>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		if let Some(min) = &self.min {
			match item.compare(min) {
				Some(Ordering::Greater) => {}
				Some(Ordering::Equal) if !self.exclusive_min => {}
				_ => return Err(Error::TooSmall.into()),
			}
		}

		if let Some(max) = &self.max {
			match item.compare(max) {
				Some(Ordering::Less) => {}
				Some(Ordering::Equal) if !self.exclusive_max => {}
				_ => return Err(Error::TooLarge.into()),
			}
		}

		Ok(())
	}
}

// this will never be used because the only time Unset is used iw with
// Option::<Unset>::None
impl<T: ?Sized> Compare<Unset> for T {
	#[inline]
	fn compare(&self, _: &Unset) -> Option<Ordering> {
		None
	}
}

impl<T: ?Sized> Compare<T> for T
where
	T: PartialOrd,
{
	#[inline]
	fn compare(&self, other: &T) -> Option<Ordering> {
		self.partial_cmp(other)
	}
}

impl<T: ?Sized> Compare<&T> for T
where
	T: PartialOrd,
{
	#[inline]
	fn compare(&self, other: &&T) -> Option<Ordering> {
		self.partial_cmp(other)
	}
}

impl Compare<&str> for Cow<'_, str> {
	#[inline]
	fn compare(&self, other: &&str) -> Option<Ordering> {
		AsRef::as_ref(self).partial_cmp(*other)
	}
}

impl Compare<Cow<'_, str>> for str {
	#[inline]
	fn compare(&self, other: &Cow<'_, str>) -> Option<Ordering> {
		self.partial_cmp(AsRef::as_ref(other))
	}
}

impl Compare<&&str> for &'_ Cow<'_, str> {
	#[inline]
	fn compare(&self, other: &&&str) -> Option<Ordering> {
		AsRef::as_ref(self).partial_cmp(other)
	}
}

impl Compare<&&str> for &'_ String {
	#[inline]
	fn compare(&self, other: &&&str) -> Option<Ordering> {
		self.as_str().partial_cmp(other)
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::Rule;

	#[test]
	fn test_integer_range() {
		for n in 1u32..=10 {
			let rule = RangeRule::new().min(&1).max(&10);
			assert!(rule.validate(&(), &n).is_ok());
		}

		let rule = RangeRule::new().exclusive_min(&0);
		assert!(rule.validate(&(), &0).is_err());

		let rule = RangeRule::new().exclusive_max(&10);
		assert!(rule.validate(&(), &11).is_err());

		let rule = RangeRule::new().min(&1);
		assert!(rule.validate(&(), &0).is_err());

		let rule = RangeRule::new().max(&10);
		assert!(rule.validate(&(), &11).is_err());
	}

	#[test]
	fn test_float_range() {
		for n in 1..=10 {
			let n = f64::from(n);

			let rule = RangeRule::new().min(&1.0).max(&10.0);
			assert!(rule.validate(&(), &n).is_ok());
		}

		let rule = RangeRule::new().exclusive_min(&0.0);
		assert!(rule.validate(&(), &0.0).is_err());

		let rule = RangeRule::new().exclusive_max(&10.0);
		assert!(rule.validate(&(), &11.0).is_err());

		let rule = RangeRule::new().min(&1.0);
		assert!(rule.validate(&(), &0.0).is_err());

		let rule = RangeRule::new().max(&10.0);
		assert!(rule.validate(&(), &11.0).is_err());
	}

	#[test]
	fn test_string_range() {
		let rule = RangeRule::new().min("hello").max("world");
		assert!(rule.validate(&(), "hello").is_ok());

		let rule = RangeRule::new().min("world");
		assert!(rule.validate(&(), "hello").is_err());

		let rule = RangeRule::new().max("hello");
		assert!(rule.validate(&(), "world").is_err());

		let rule = RangeRule::new().exclusive_min("hello");
		assert!(rule.validate(&(), "hello").is_err());

		let rule = RangeRule::new().exclusive_max("world");
		assert!(rule.validate(&(), "world").is_err());

		let rule = RangeRule::new().min("worlds");
		assert!(rule.validate(&(), "world").is_err());

		let rule = RangeRule::new().max("worl");
		assert!(rule.validate(&(), "world").is_err());
	}
}
