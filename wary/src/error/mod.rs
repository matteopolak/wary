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
	#[error("value is not alphanumeric")]
	Alphanumeric,
	#[error("value is not ascii")]
	Ascii,
	#[error(transparent)]
	Addr(#[from] rule::addr::Error),
	#[error("found non-lowercase character at position {position}")]
	Lowercase { position: usize },
	#[error("found non-uppercase character at position {position}")]
	Uppercase { position: usize },
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
	Email(#[from] email_address::Error),
	#[cfg(feature = "url")]
	#[error(transparent)]
	Url(#[from] url::ParseError),
	#[error(transparent)]
	Length(#[from] rule::length::Error),
	#[error(transparent)]
	Range(#[from] rule::range::Error),
	#[cfg(feature = "semver")]
	#[error("value is not a valid semver version")]
	Semver,
	#[cfg(feature = "regex")]
	#[error(transparent)]
	Regex(#[from] rule::regex::Error),
	#[error(transparent)]
	Required(#[from] rule::required::Error),
	#[error("{code}")]
	Custom {
		code: &'static str,
		#[cfg(feature = "alloc")]
		message: Option<Cow<'static, str>>,
		#[cfg(not(feature = "alloc"))]
		message: Option<&'static str>,
	},
}

impl Error {
	#[must_use]
	pub fn new(code: &'static str) -> Self {
		Self::Custom {
			code,
			message: None,
		}
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
}

#[cfg(not(feature = "alloc"))]
impl Report {
	pub fn push(&mut self, path: Path, error: Error) {
		if self.len == self.errors.len() {
			return;
		}

		self.errors[self.len] = Some((path, error));
		self.len += 1;
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.len == 0
	}
}

#[cfg(all(feature = "serde", feature = "alloc"))]
impl serde::Serialize for Report {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		use serde::ser::SerializeSeq;

		let mut seq = serializer.serialize_seq(Some(self.errors.len()))?;

		for (path, error) in &self.errors {
			seq.serialize_element(&(path, error))?;
		}

		seq.end()
	}
}

#[cfg(all(feature = "serde", not(feature = "alloc")))]
impl serde::Serialize for Report {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_unit()
	}
}
