use crate::{Error, Validate};

#[doc(hidden)]
pub type Rule<T> = LengthRule<T>;

#[derive(Debug, thiserror::Error)]
pub enum LengthError {
	#[error("Expected length of at least {min}, found {actual}")]
	TooShort { min: usize, actual: usize },
	#[error("Expected length of at most {max}, found {actual}")]
	TooLong { max: usize, actual: usize },
}

#[diagnostic::on_unimplemented(note = "For string types, wrap the value in `Bytes` or `Chars` to \
                                       get the byte or character length, respectively.")]
pub trait Length {
	fn length(&self) -> usize;
}

/// Length in bytes for string-like containers.
pub struct Bytes<T>(pub T);

/// Length in characters for string-like containers that hold UTF-8.
pub struct Chars<T>(pub T);

/// Length in UTF-16 code units.
pub struct CodeUnits<T>(pub T);

/// Length in grapheme clusters.
#[cfg(feature = "graphemes")]
pub struct Graphemes<T>(pub T);

pub struct LengthRule<T> {
	min: usize,
	max: usize,
	exclusive_min: bool,
	exclusive_max: bool,
	inner: T,
}

impl<T> LengthRule<T> {
	pub fn new(inner: T) -> Self {
		Self {
			min: usize::MIN,
			max: usize::MAX,
			exclusive_min: false,
			exclusive_max: false,
			inner,
		}
	}

	pub fn chars(self) -> LengthRule<Chars<T>>
	where
		Chars<T>: Length,
	{
		LengthRule {
			min: self.min,
			max: self.max,
			exclusive_min: self.exclusive_min,
			exclusive_max: self.exclusive_max,
			inner: Chars(self.inner),
		}
	}

	pub fn bytes(self) -> LengthRule<Bytes<T>>
	where
		Bytes<T>: Length,
	{
		LengthRule {
			min: self.min,
			max: self.max,
			exclusive_min: self.exclusive_min,
			exclusive_max: self.exclusive_max,
			inner: Bytes(self.inner),
		}
	}

	pub fn code_units(self) -> LengthRule<CodeUnits<T>>
	where
		CodeUnits<T>: Length,
	{
		LengthRule {
			min: self.min,
			max: self.max,
			exclusive_min: self.exclusive_min,
			exclusive_max: self.exclusive_max,
			inner: CodeUnits(self.inner),
		}
	}

	#[cfg(feature = "graphemes")]
	pub fn graphemes(self) -> LengthRule<Graphemes<T>>
	where
		Graphemes<T>: Length,
	{
		LengthRule {
			min: self.min,
			max: self.max,
			exclusive_min: self.exclusive_min,
			exclusive_max: self.exclusive_max,
			inner: Graphemes(self.inner),
		}
	}

	pub fn min(mut self, min: usize) -> Self {
		self.min = min;
		self.exclusive_min = false;
		self
	}

	pub fn max(mut self, max: usize) -> Self {
		self.max = max;
		self.exclusive_max = false;
		self
	}

	pub fn exclusive_min(mut self, min: usize) -> Self {
		self.min = min;
		self.exclusive_min = true;
		self
	}

	pub fn exclusive_max(mut self, max: usize) -> Self {
		self.max = max;
		self.exclusive_max = true;
		self
	}
}

impl<T> Validate for LengthRule<T>
where
	T: Length,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context) -> Result<(), Error> {
		let len = self.inner.length();

		if len < self.min {
			return Err(
				LengthError::TooShort {
					min: self.min,
					actual: len,
				}
				.into(),
			);
		}

		if len > self.max {
			return Err(
				LengthError::TooLong {
					max: self.max,
					actual: len,
				}
				.into(),
			);
		}

		Ok(())
	}
}

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
		self.as_ref().length()
	}
}

impl<T> Length for Bytes<T>
where
	T: AsRef<[u8]>,
{
	#[inline]
	fn length(&self) -> usize {
		self.0.as_ref().len()
	}
}

impl<T> Length for Chars<T>
where
	T: AsRef<str>,
{
	#[inline]
	fn length(&self) -> usize {
		self.0.as_ref().chars().count()
	}
}

impl<T> Length for CodeUnits<T>
where
	T: AsRef<str>,
{
	#[inline]
	fn length(&self) -> usize {
		self.0.as_ref().encode_utf16().count()
	}
}

impl<T> Length for Graphemes<T>
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
	use super::*;

	#[test]
	fn test_string_length() {
		let rule = LengthRule::new("hello").bytes().min(5).max(5);
		assert!(rule.validate(&()).is_ok());

		let rule = LengthRule::new("hello").bytes().min(6).max(6);
		assert!(rule.validate(&()).is_err());

		let rule = LengthRule::new("hello").chars().min(5).max(5);
		assert!(rule.validate(&()).is_ok());

		let rule = LengthRule::new("hello").chars().min(6).max(6);
		assert!(rule.validate(&()).is_err());

		let rule = LengthRule::new("😊").chars().min(1).max(1);
		assert!(rule.validate(&()).is_ok());

		let rule = LengthRule::new("😊").bytes().min(1).max(1);
		assert!(rule.validate(&()).is_err());
	}

	#[test]
	fn test_slice_length() {
		let rule = LengthRule::new([1u8, 2, 3, 4, 5].as_slice()).min(5).max(5);
		assert!(rule.validate(&()).is_ok());

		let rule = LengthRule::new([1, 2, 3, 4, 5].as_slice()).min(6).max(6);
		assert!(rule.validate(&()).is_err());

		let rule = LengthRule::new(vec![1, 2, 3, 4, 5]).min(5).max(5);
		assert!(rule.validate(&()).is_ok());

		let rule = LengthRule::new(&[1, 2, 3, 4, 5]).min(6).max(6);
		assert!(rule.validate(&()).is_err());
	}
}
