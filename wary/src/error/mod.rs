mod path;

pub use path::Path;

#[cfg(feature = "alloc")]
use crate::alloc::{borrow::Cow, vec::Vec};
use crate::options::rule;

#[derive(Debug, thiserror::Error, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[non_exhaustive]
pub enum Error {
	#[error(transparent)]
	Alphanumeric(#[from] rule::alphanumeric::Error),
	#[error(transparent)]
	Ascii(#[from] rule::ascii::Error),
	#[error(transparent)]
	Addr(#[from] rule::addr::Error),
	#[error(transparent)]
	Lowercase(#[from] rule::lowercase::Error),
	#[error(transparent)]
	Uppercase(#[from] rule::uppercase::Error),
	#[error(transparent)]
	Contains(#[from] rule::contains::Error),
	#[error(transparent)]
	Prefix(#[from] rule::prefix::Error),
	#[error(transparent)]
	Suffix(#[from] rule::suffix::Error),
	#[error(transparent)]
	Equals(#[from] rule::equals::Error),
	#[cfg(feature = "email")]
	#[error(transparent)]
	Email(#[from] rule::email::Error),
	#[cfg(feature = "url")]
	#[error(transparent)]
	Url(#[from] rule::url::Error),
	#[error(transparent)]
	Length(#[from] rule::length::Error),
	#[error(transparent)]
	Range(#[from] rule::range::Error),
	#[cfg(feature = "semver")]
	#[error(transparent)]
	Semver(#[from] rule::semver::Error),
	#[cfg(feature = "regex")]
	#[error(transparent)]
	Regex(#[from] rule::regex::Error),
	#[error(transparent)]
	Required(#[from] rule::required::Error),
	#[cfg(feature = "uuid")]
	#[error(transparent)]
	Uuid(#[from] rule::uuid::Error),
	#[cfg(feature = "credit_card")]
	#[error(transparent)]
	CreditCard(#[from] rule::credit_card::Error),
	#[error("{code}")]
	Custom {
		code: &'static str,
		#[cfg(feature = "alloc")]
		message: Option<Cow<'static, str>>,
		#[cfg(not(feature = "alloc"))]
		message: Option<&'static str>,
	},
}

#[allow(dead_code)]
impl Error {
	#[must_use]
	pub fn new(code: &'static str) -> Self {
		Self::Custom {
			code,
			message: None,
		}
	}

	#[must_use]
	pub fn is_custom(&self) -> bool {
		matches!(self, Self::Custom { .. })
	}

	#[cfg(feature = "alloc")]
	pub fn with_message(code: &'static str, message: impl Into<Cow<'static, str>>) -> Self {
		Self::Custom {
			code,
			message: Some(message.into()),
		}
	}

	#[cfg(not(feature = "alloc"))]
	pub fn with_message(code: &'static str, message: &'static str) -> Self {
		Self::Custom {
			code,
			message: Some(message),
		}
	}

	#[must_use]
	pub(crate) fn code(&self) -> &'static str {
		match self {
			Self::Alphanumeric(error) => error.code(),
			Self::Ascii(error) => error.code(),
			Self::Addr(error) => error.code(),
			Self::Lowercase(error) => error.code(),
			Self::Uppercase(error) => error.code(),
			Self::Contains(error) => error.code(),
			Self::Prefix(error) => error.code(),
			Self::Suffix(error) => error.code(),
			Self::Equals(error) => error.code(),
			#[cfg(feature = "email")]
			Self::Email(error) => error.code(),
			#[cfg(feature = "url")]
			Self::Url(error) => error.code(),
			Self::Length(error) => error.code(),
			Self::Range(error) => error.code(),
			#[cfg(feature = "semver")]
			Self::Semver(error) => error.code(),
			#[cfg(feature = "regex")]
			Self::Regex(error) => error.code(),
			Self::Required(error) => error.code(),
			#[cfg(feature = "uuid")]
			Self::Uuid(error) => error.code(),
			#[cfg(feature = "credit_card")]
			Self::CreditCard(error) => error.code(),
			Self::Custom { code, .. } => code,
		}
	}

	#[cfg(feature = "alloc")]
	pub(crate) fn message(&self) -> Option<Cow<str>> {
		Some(match self {
			Self::Alphanumeric(error) => error.message(),
			Self::Ascii(error) => error.message(),
			Self::Addr(error) => error.message(),
			Self::Lowercase(error) => error.message(),
			Self::Uppercase(error) => error.message(),
			Self::Contains(error) => error.message(),
			Self::Prefix(error) => error.message(),
			Self::Suffix(error) => error.message(),
			Self::Equals(error) => error.message(),
			#[cfg(feature = "email")]
			Self::Email(error) => error.message(),
			#[cfg(feature = "url")]
			Self::Url(error) => error.message(),
			Self::Length(error) => error.message(),
			Self::Range(error) => error.message(),
			#[cfg(feature = "semver")]
			Self::Semver(error) => error.message(),
			#[cfg(feature = "regex")]
			Self::Regex(error) => error.message(),
			Self::Required(error) => error.message(),
			#[cfg(feature = "uuid")]
			Self::Uuid(error) => error.message(),
			#[cfg(feature = "credit_card")]
			Self::CreditCard(error) => error.message(),
			Self::Custom { message, .. } => return message.as_deref().map(Cow::Borrowed),
		})
	}

	#[cfg(not(feature = "alloc"))]
	pub(crate) fn message(&self) -> Option<&'static str> {
		Some(match self {
			Self::Alphanumeric(error) => error.message(),
			Self::Ascii(error) => error.message(),
			Self::Addr(error) => error.message(),
			Self::Lowercase(error) => error.message(),
			Self::Uppercase(error) => error.message(),
			Self::Contains(error) => error.message(),
			Self::Prefix(error) => error.message(),
			Self::Suffix(error) => error.message(),
			Self::Equals(error) => error.message(),
			#[cfg(feature = "email")]
			Self::Email(error) => error.message(),
			#[cfg(feature = "url")]
			Self::Url(error) => error.message(),
			Self::Length(error) => error.message(),
			Self::Range(error) => error.message(),
			#[cfg(feature = "semver")]
			Self::Semver(error) => error.message(),
			#[cfg(feature = "regex")]
			Self::Regex(error) => error.message(),
			Self::Required(error) => error.message(),
			#[cfg(feature = "uuid")]
			Self::Uuid(error) => error.message(),
			#[cfg(feature = "credit_card")]
			Self::CreditCard(error) => error.message(),
			Self::Custom { message, .. } => return *message,
		})
	}
}

#[derive(Debug, Default)]
pub struct Report {
	#[cfg(feature = "alloc")]
	errors: Vec<(Path, Error)>,
	#[cfg(not(feature = "alloc"))]
	errors: [Option<(Path, Error)>; 1],
	#[cfg(not(feature = "alloc"))]
	len: usize,
}

#[cfg(feature = "alloc")]
impl Report {
	pub fn push(&mut self, path: Path, error: Error) {
		self.errors.push((path, error));
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.errors.is_empty()
	}

	#[must_use]
	pub fn len(&self) -> usize {
		self.errors.len()
	}

	pub fn clear(&mut self) {
		self.errors.clear();
	}

	pub fn extend(&mut self, other: Self) {
		self.errors.extend(other.errors);
	}
}

#[cfg(not(feature = "alloc"))]
impl Report {
	pub fn push(&mut self, path: Path, error: Error) {
		if self.len < self.errors.len() {
			self.errors[self.len] = Some((path, error));
		}

		self.len += 1;
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.len == 0
	}

	#[must_use]
	pub fn len(&self) -> usize {
		self.len
	}

	pub fn clear(&mut self) {
		for i in 0..self.len.min(self.errors.len()) {
			self.errors[i] = None;
		}

		self.len = 0;
	}

	pub fn extend(&mut self, other: Self) {
		for error in other.errors {
			if self.len < self.errors.len() {
				self.errors[self.len] = error;
			}

			self.len += 1;
		}
	}
}

#[cfg(feature = "serde")]
mod ser {
	use super::*;

	#[derive(serde::Serialize)]
	struct Inner<'d> {
		path: &'d Path,
		code: &'static str,
		message: Option<Cow<'d, str>>,
		#[serde(skip_serializing_if = "Error::is_custom")]
		detail: &'d Error,
	}

	#[cfg(feature = "alloc")]
	impl serde::Serialize for Report {
		fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
		where
			S: serde::Serializer,
		{
			use serde::ser::SerializeSeq;

			let mut seq = serializer.serialize_seq(Some(self.errors.len()))?;

			for (path, error) in &self.errors {
				let detail = Inner {
					path,
					code: error.code(),
					message: error.message(),
					detail: error,
				};

				seq.serialize_element(&detail)?;
			}

			seq.end()
		}
	}

	#[cfg(not(feature = "alloc"))]
	impl serde::Serialize for Report {
		fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
		where
			S: serde::Serializer,
		{
			use serde::ser::SerializeSeq;

			let mut seq = serializer.serialize_seq(Some(self.len))?;

			for i in 0..self.len {
				if let Some((path, error)) = &self.errors[i] {
					let detail = Inner {
						path,
						code: error.code(),
						message: error.message(),
						detail: error,
					};

					seq.serialize_element(&detail)?;
				}
			}
		}
	}
}
