//! Rule for length validation.
//!
//! See [`LengthRule`] for more information.

use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule<Mode> = LengthRule<Mode>;

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
	#[error("Expected length of at least {min}, found {actual}")]
	TooShort {
		min: usize,
		actual: usize,
		exclusive: bool,
	},
	#[error("Expected length of at most {max}, found {actual}")]
	TooLong {
		max: usize,
		actual: usize,
		exclusive: bool,
	},
}

/// Rule for length validation.
///
/// # Example
///
/// ```
/// use wary::{Wary, Validate};
///
/// #[derive(Wary)]
/// struct Person {
///   #[validate(length(chars, 5..=10))]
///   name: String,
///   #[validate(length(5..10))]
///   numbers: Vec<u8>,
///   #[validate(length(bytes, 1..))]
///   greeting: String,
/// }
///
/// let person = Person {
///   name: "hello".into(),
///   numbers: vec![1, 2, 3, 4, 5],
///   greeting: "hi".into(),
/// };
///
/// assert!(person.validate(&()).is_ok());
///
/// let person = Person {
///   name: "hello".into(),
///   numbers: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
///   greeting: "hi".into(),
/// };
///
/// assert!(person.validate(&()).is_err());
/// ```
#[must_use]
pub struct LengthRule<Mode> {
	min: usize,
	max: usize,
	exclusive_min: bool,
	exclusive_max: bool,
	mode: PhantomData<Mode>,
}

pub struct Bytes;
pub struct Chars;
pub struct CodeUnits;
#[cfg(feature = "graphemes")]
pub struct Graphemes;

impl LengthRule<Unset> {
	#[inline]
	pub const fn new() -> Self {
		Self {
			min: usize::MIN,
			max: usize::MAX,
			exclusive_min: false,
			exclusive_max: false,
			mode: PhantomData,
		}
	}

	/// Set the length mode to count in UTF-8 characters.
	#[inline]
	pub const fn chars(self) -> LengthRule<Chars> {
		LengthRule {
			min: self.min,
			max: self.max,
			exclusive_min: self.exclusive_min,
			exclusive_max: self.exclusive_max,
			mode: PhantomData,
		}
	}

	/// Set the length mode to count in bytes.
	#[inline]
	pub const fn bytes(self) -> LengthRule<Bytes> {
		LengthRule {
			min: self.min,
			max: self.max,
			exclusive_min: self.exclusive_min,
			exclusive_max: self.exclusive_max,
			mode: PhantomData,
		}
	}

	/// Set the length mode to count in UTF-16 code units.
	#[inline]
	pub const fn code_units(self) -> LengthRule<CodeUnits> {
		LengthRule {
			min: self.min,
			max: self.max,
			exclusive_min: self.exclusive_min,
			exclusive_max: self.exclusive_max,
			mode: PhantomData,
		}
	}

	/// Set the length mode to count in grapheme clusters.
	#[cfg(feature = "graphemes")]
	#[inline]
	pub const fn graphemes(self) -> LengthRule<Graphemes> {
		LengthRule {
			min: self.min,
			max: self.max,
			exclusive_min: self.exclusive_min,
			exclusive_max: self.exclusive_max,
			mode: PhantomData,
		}
	}
}

impl<Mode> LengthRule<Mode> {
	/// Set the minimum length (inclusive).
	#[inline]
	pub const fn min(mut self, min: usize) -> Self {
		self.min = min;
		self.exclusive_min = false;
		self
	}

	/// Set the maximum length (inclusive).
	#[inline]
	pub const fn max(mut self, max: usize) -> Self {
		self.max = max;
		self.exclusive_max = false;
		self
	}

	/// Set the minimum length (exclusive).
	#[inline]
	pub const fn exclusive_min(mut self, min: usize) -> Self {
		self.min = min;
		self.exclusive_min = true;
		self
	}

	/// Set the maximum length (exclusive).
	#[inline]
	pub const fn exclusive_max(mut self, max: usize) -> Self {
		self.max = max;
		self.exclusive_max = true;
		self
	}

	#[inline]
	fn check(&self, len: usize) -> Result<()> {
		if len < self.min || len == self.min && self.exclusive_min {
			return Err(
				Error::TooShort {
					min: self.min,
					actual: len,
					exclusive: self.exclusive_min,
				}
				.into(),
			);
		}

		if len > self.max || len == self.max && self.exclusive_max {
			return Err(
				Error::TooLong {
					max: self.max,
					actual: len,
					exclusive: self.exclusive_max,
				}
				.into(),
			);
		}

		Ok(())
	}
}

impl<I: ?Sized> crate::Rule<I> for LengthRule<Unset>
where
	I: Length,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let len = item.length();

		self.check(len)
	}
}

impl<I: ?Sized> crate::Rule<I> for LengthRule<Bytes>
where
	for<'d> BytesLength<'d, I>: Length,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let len = BytesLength(item).length();

		self.check(len)
	}
}

impl<I: ?Sized> crate::Rule<I> for LengthRule<Chars>
where
	for<'d> CharsLength<'d, I>: Length,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let len = CharsLength(item).length();

		self.check(len)
	}
}

impl<I: ?Sized> crate::Rule<I> for LengthRule<CodeUnits>
where
	for<'d> CodeUnitsLength<'d, I>: Length,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let len = CodeUnitsLength(item).length();

		self.check(len)
	}
}

#[cfg(feature = "graphemes")]
impl<I: ?Sized> crate::Rule<I> for LengthRule<Graphemes>
where
	for<'d> GraphemesLength<'d, I>: Length,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let len = GraphemesLength(item).length();

		self.check(len)
	}
}

#[diagnostic::on_unimplemented(note = "For string types, wrap the value in `Bytes` or `Chars` to \
                                       get the byte or character length, respectively.")]
pub trait Length {
	fn length(&self) -> usize;
}

/// Length in bytes for string-like containers.
pub struct BytesLength<'d, T: ?Sized>(&'d T);

/// Length in characters for string-like containers that hold UTF-8.
pub struct CharsLength<'d, T: ?Sized>(&'d T);

/// Length in UTF-16 code units.
pub struct CodeUnitsLength<'d, T: ?Sized>(&'d T);

/// Length in grapheme clusters.
#[cfg(feature = "graphemes")]
pub struct GraphemesLength<'d, T: ?Sized>(&'d T);

impl<T> Length for T
where
	T: AsSlice,
{
	fn length(&self) -> usize {
		self.as_slice().len()
	}
}

impl<T: ?Sized> Length for BytesLength<'_, T>
where
	T: AsRef<[u8]>,
{
	#[inline]
	fn length(&self) -> usize {
		self.0.as_ref().len()
	}
}

impl<T: ?Sized> Length for CharsLength<'_, T>
where
	T: AsRef<str>,
{
	#[inline]
	fn length(&self) -> usize {
		self.0.as_ref().chars().count()
	}
}

impl<T: ?Sized> Length for CodeUnitsLength<'_, T>
where
	T: AsRef<str>,
{
	#[inline]
	fn length(&self) -> usize {
		self.0.as_ref().encode_utf16().count()
	}
}

#[cfg(feature = "graphemes")]
impl<T: ?Sized> Length for GraphemesLength<'_, T>
where
	T: AsRef<str>,
{
	#[inline]
	fn length(&self) -> usize {
		use unicode_segmentation::UnicodeSegmentation;

		self.0.as_ref().graphemes(true).count()
	}
}

#[cfg(test)]
mod test {
	use super::LengthRule;
	use crate::toolbox::test::*;

	#[test]
	fn test_string_length() {
		let rule = LengthRule::new().bytes().min(5).max(5);
		assert!(rule.validate(&(), "hello").is_ok());

		let rule = LengthRule::new().bytes().min(6).max(6);
		assert!(rule.validate(&(), "hello").is_err());

		let rule = LengthRule::new().chars().min(5).max(5);
		assert!(rule.validate(&(), "hello").is_ok());

		let rule = LengthRule::new().chars().min(6).max(6);
		assert!(rule.validate(&(), "hello").is_err());

		let rule = LengthRule::new().chars().min(1).max(1);
		assert!(rule.validate(&(), "😊").is_ok());

		let rule = LengthRule::new().bytes().min(1).max(1);
		assert!(rule.validate(&(), "😊").is_err());
	}

	#[test]
	fn test_slice_length() {
		let rule = LengthRule::new().min(5).max(5);
		assert!(rule.validate(&(), &[1u8, 2, 3, 4, 5].as_slice()).is_ok());

		let rule = LengthRule::new().min(6).max(6);
		assert!(rule.validate(&(), &[1, 2, 3, 4, 5].as_slice()).is_err());

		let rule = LengthRule::new().min(5).max(5);
		assert!(rule.validate(&(), &vec![1, 2, 3, 4, 5]).is_ok());

		let rule = LengthRule::new().min(6).max(6);
		assert!(rule.validate(&(), &[1, 2, 3, 4, 5]).is_err());
	}
}
