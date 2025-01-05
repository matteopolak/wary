//! Rule for checking if a value is within a range.
//!
//! See [`RangeRule`] for more information.

use core::cmp::Ordering;

use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule<Min, Max> = RangeRule<Min, Max>;

#[derive(Debug, thiserror::Error, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case", tag = "code"))]
pub enum Error {
	#[error("value is too small")]
	TooSmall,
	#[error("value is too large")]
	TooLarge,
}

impl Error {
	#[must_use]
	pub fn code(&self) -> &'static str {
		match self {
			Self::TooSmall => "too_small",
			Self::TooLarge => "too_large",
		}
	}

	#[cfg(feature = "alloc")]
	#[must_use]
	pub fn message(&self) -> Cow<'static, str> {
		match self {
			Self::TooSmall => "value is too small".into(),
			Self::TooLarge => "value is too large".into(),
		}
	}

	#[cfg(not(feature = "alloc"))]
	pub fn message(&self) -> &'static str {
		match self {
			Self::TooSmall => "value is too small",
			Self::TooLarge => "value is too large",
		}
	}
}

pub trait Compare<B: ?Sized = Self> {
	fn compare(&self, other: &B) -> Option<Ordering>;
}

/// Rule for checking if a value is within a range.
///
/// # Example
///
/// ```
/// use wary::{Wary, Validate};
///
/// #[derive(Wary)]
/// struct Number {
///   #[validate(range(min = 1, max = 10))]
///   n: u32,
///   #[validate(range(min = 1, exclusive_max = 10))]
///   n_exclusive_max: i32,
///   #[validate(range(exclusive_min = 1, max = 10))]
///   n_exclusive_min: u8,
/// }
///
/// let number = Number { n: 5, n_exclusive_max: 9, n_exclusive_min: 2 };
/// assert!(number.validate(&()).is_ok());
///
/// let number = Number { n: 0, n_exclusive_max: 10, n_exclusive_min: 1 };
/// assert!(number.validate(&()).is_err());
/// ```
#[must_use]
pub struct RangeRule<Min, Max> {
	min: Option<Min>,
	max: Option<Max>,
	exclusive_min: bool,
	exclusive_max: bool,
}

impl RangeRule<Unset, Unset> {
	#[inline]
	pub const fn new() -> Self {
		RangeRule {
			min: None,
			max: None,
			exclusive_min: false,
			exclusive_max: false,
		}
	}
}

impl<Max> RangeRule<Unset, Max> {
	/// Set the minimum value (inclusive).
	#[inline]
	pub fn min<Min>(self, min: Min) -> RangeRule<Min, Max> {
		RangeRule {
			min: Some(min),
			max: self.max,
			exclusive_min: false,
			exclusive_max: self.exclusive_max,
		}
	}

	/// Set the minimum value (exclusive).
	#[inline]
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
	/// Set the maximum value (inclusive).
	#[inline]
	pub fn max<Max>(self, max: Max) -> RangeRule<Min, Max> {
		RangeRule {
			min: self.min,
			max: Some(max),
			exclusive_min: self.exclusive_min,
			exclusive_max: false,
		}
	}

	/// Set the maximum value (exclusive).
	#[inline]
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

#[cfg(feature = "alloc")]
impl Compare<&str> for Cow<'_, str> {
	#[inline]
	fn compare(&self, other: &&str) -> Option<Ordering> {
		AsRef::as_ref(self).partial_cmp(*other)
	}
}

#[cfg(feature = "alloc")]
impl Compare<Cow<'_, str>> for str {
	#[inline]
	fn compare(&self, other: &Cow<'_, str>) -> Option<Ordering> {
		self.partial_cmp(AsRef::as_ref(other))
	}
}

#[cfg(feature = "alloc")]
impl Compare<&&str> for String {
	#[inline]
	fn compare(&self, other: &&&str) -> Option<Ordering> {
		self.as_str().partial_cmp(**other)
	}
}

#[cfg(feature = "alloc")]
impl Compare<&&str> for Cow<'_, str> {
	#[inline]
	fn compare(&self, other: &&&str) -> Option<Ordering> {
		AsRef::as_ref(self).partial_cmp(**other)
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
