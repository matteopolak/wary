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
	fn length(&self) -> usize;
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
		let len = self.inner.length();

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

impl<T> Validate for LengthRule<Option<T>>
where
	T: Length,
{
	type Context = ();

	fn validate(&self, ctx: &Self::Context) -> Result<(), Error> {
		if let Some(inner) = &self.inner {
			LengthRule::new(inner).validate(ctx)
		} else {
			Ok(())
		}
	}
}

impl<T> Length for [T] {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<T> Length for &T
where
	T: Length,
{
	fn length(&self) -> usize {
		(**self).length()
	}
}

impl<T> Length for Vec<T> {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<T> Length for Bytes<T>
where
	T: AsRef<[u8]>,
{
	fn length(&self) -> usize {
		self.0.as_ref().len()
	}
}

impl<T> Length for Chars<T>
where
	T: AsRef<str>,
{
	fn length(&self) -> usize {
		self.0.as_ref().chars().count()
	}
}
