use std::{
	borrow::Cow,
	ffi::{OsStr, OsString},
};

use crate::{Error, Validate};

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
	fn length(&self) -> Option<usize>;
}

/// Length in bytes for string-like containers.
pub struct Bytes<T>(pub T);

/// Length in characters for string-like containers that hold UTF-8.
pub struct Chars<T>(pub T);

pub struct LengthRule<T> {
	min: Option<usize>,
	max: Option<usize>,
	inner: T,
}

impl<T> LengthRule<T> {
	pub fn new(inner: T) -> Self {
		Self {
			min: None,
			max: None,
			inner,
		}
	}

	pub fn chars(self) -> LengthRule<Chars<T>> {
		LengthRule {
			min: self.min,
			max: self.max,
			inner: Chars(self.inner),
		}
	}

	pub fn bytes(self) -> LengthRule<Bytes<T>> {
		LengthRule {
			min: self.min,
			max: self.max,
			inner: Bytes(self.inner),
		}
	}

	pub fn min(mut self, min: usize) -> Self {
		self.min = Some(min);
		self
	}

	pub fn max(mut self, max: usize) -> Self {
		self.max = Some(max);
		self
	}
}

impl<T> Validate for LengthRule<T>
where
	T: Length,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context) -> Result<(), Error> {
		let Some(len) = self.inner.length() else {
			return Ok(());
		};

		if let Some(min) = self.min {
			if len < min {
				return Err(LengthError::TooShort { min, actual: len }.into());
			}
		}

		if let Some(max) = self.max {
			if len > max {
				return Err(LengthError::TooLong { max, actual: len }.into());
			}
		}

		Ok(())
	}
}

impl<T> Length for Option<T>
where
	T: Length,
{
	fn length(&self) -> Option<usize> {
		self.as_ref().and_then(Length::length)
	}
}

impl<T> Length for &T
where
	T: Length + ?Sized,
{
	fn length(&self) -> Option<usize> {
		(**self).length()
	}
}

impl<const N: usize, T> Length for [T; N] {
	#[inline]
	fn length(&self) -> Option<usize> {
		Some(N)
	}
}

impl<T> Length for [T] {
	#[inline]
	fn length(&self) -> Option<usize> {
		Some(self.len())
	}
}

impl<T> Length for Vec<T> {
	#[inline]
	fn length(&self) -> Option<usize> {
		self.as_slice().length()
	}
}

impl<T> Length for Bytes<Option<T>>
where
	for<'a> Bytes<&'a T>: Length,
{
	#[inline]
	fn length(&self) -> Option<usize> {
		self.0.as_ref().map(Bytes).length()
	}
}

impl Length for Bytes<&str> {
	#[inline]
	fn length(&self) -> Option<usize> {
		Some(self.0.len())
	}
}

impl Length for Bytes<String> {
	fn length(&self) -> Option<usize> {
		Bytes(self.0.as_str()).length()
	}
}

impl Length for Bytes<&String> {
	fn length(&self) -> Option<usize> {
		Bytes(self.0.as_str()).length()
	}
}

impl Length for Bytes<Cow<'_, str>> {
	fn length(&self) -> Option<usize> {
		Bytes(self.0.as_ref()).length()
	}
}

impl Length for OsStr {
	fn length(&self) -> Option<usize> {
		Some(self.len())
	}
}

impl Length for OsString {
	fn length(&self) -> Option<usize> {
		self.as_os_str().length()
	}
}

impl<T> Length for Chars<&T>
where
	T: Length,
{
	#[inline]
	fn length(&self) -> Option<usize> {
		self.0.length()
	}
}

impl Length for Chars<&str> {
	#[inline]
	fn length(&self) -> Option<usize> {
		Some(self.0.chars().count())
	}
}

impl Length for Chars<Option<&str>> {
	fn length(&self) -> Option<usize> {
		self.0.map(Chars).length()
	}
}

impl Length for Chars<String> {
	fn length(&self) -> Option<usize> {
		Chars(self.0.as_str()).length()
	}
}

impl Length for Chars<&String> {
	fn length(&self) -> Option<usize> {
		Chars(self.0.as_str()).length()
	}
}

impl Length for Chars<Option<String>> {
	fn length(&self) -> Option<usize> {
		self.0.as_ref().map(Chars).length()
	}
}

impl Length for Chars<Option<&String>> {
	fn length(&self) -> Option<usize> {
		self.0.map(Chars).length()
	}
}

impl Length for Chars<Cow<'_, str>> {
	fn length(&self) -> Option<usize> {
		Chars(self.0.as_ref()).length()
	}
}

impl Length for Chars<Option<Cow<'_, str>>> {
	fn length(&self) -> Option<usize> {
		self.0.as_deref().map(Chars).length()
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

	#[test]
	fn test_option_length() {
		let rule = LengthRule::new(Some("hello")).chars().min(5).max(5);
		assert!(rule.validate(&()).is_ok());

		let rule = LengthRule::new(Some("hello")).chars().min(6).max(6);
		assert!(rule.validate(&()).is_err());

		let rule = LengthRule::new(None::<&str>).chars().min(5).max(5);
		assert!(rule.validate(&()).is_ok());
	}
}
