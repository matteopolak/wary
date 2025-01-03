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
	#[must_use]
	pub fn new() -> Self {
		Self {
			min: usize::MIN,
			max: usize::MAX,
			exclusive_min: false,
			exclusive_max: false,
			mode: PhantomData,
		}
	}

	#[must_use]
	pub fn chars(self) -> LengthRule<Chars> {
		LengthRule {
			min: self.min,
			max: self.max,
			exclusive_min: self.exclusive_min,
			exclusive_max: self.exclusive_max,
			mode: PhantomData,
		}
	}

	#[must_use]
	pub fn bytes(self) -> LengthRule<Bytes> {
		LengthRule {
			min: self.min,
			max: self.max,
			exclusive_min: self.exclusive_min,
			exclusive_max: self.exclusive_max,
			mode: PhantomData,
		}
	}

	#[must_use]
	pub fn code_units(self) -> LengthRule<CodeUnits> {
		LengthRule {
			min: self.min,
			max: self.max,
			exclusive_min: self.exclusive_min,
			exclusive_max: self.exclusive_max,
			mode: PhantomData,
		}
	}

	#[cfg(feature = "graphemes")]
	#[must_use]
	pub fn graphemes(self) -> LengthRule<Graphemes> {
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
	#[must_use]
	pub fn min(mut self, min: usize) -> Self {
		self.min = min;
		self.exclusive_min = false;
		self
	}

	#[must_use]
	pub fn max(mut self, max: usize) -> Self {
		self.max = max;
		self.exclusive_max = false;
		self
	}

	#[must_use]
	pub fn exclusive_min(mut self, min: usize) -> Self {
		self.min = min;
		self.exclusive_min = true;
		self
	}

	#[must_use]
	pub fn exclusive_max(mut self, max: usize) -> Self {
		self.max = max;
		self.exclusive_max = true;
		self
	}

	fn check(&self, len: usize) -> Result<()> {
		if len < self.min {
			return Err(
				Error::TooShort {
					min: self.min,
					actual: len,
					exclusive: self.exclusive_min,
				}
				.into(),
			);
		}

		if len > self.max {
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

impl<T> Length for &T
where
	T: Length + ?Sized,
{
	fn length(&self) -> usize {
		(**self).length()
	}
}

impl<const N: usize, T> Length for [T; N] {
	#[inline]
	fn length(&self) -> usize {
		N
	}
}

impl<T> Length for [T] {
	#[inline]
	fn length(&self) -> usize {
		self.len()
	}
}

impl<T> Length for Vec<T> {
	#[inline]
	fn length(&self) -> usize {
		self.as_slice().length()
	}
}

impl<T> Length for Box<[T]> {
	#[inline]
	fn length(&self) -> usize {
		AsRef::as_ref(self).length()
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
