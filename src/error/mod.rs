mod path;

use std::borrow::Cow;

pub use path::Path;

use crate::options::rule;

#[derive(Debug, thiserror::Error)]
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
	#[error(transparent)]
	Semver(#[from] semver::Error),
	#[cfg(feature = "regex")]
	#[error(transparent)]
	Regex(#[from] rule::regex::Error),
	#[error("{code}")]
	Custom {
		code: &'static str,
		message: Option<Cow<'static, str>>,
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

	pub fn with_message(code: &'static str, message: impl Into<Cow<'static, str>>) -> Self {
		Self::Custom {
			code,
			message: Some(message.into()),
		}
	}
}

#[derive(Debug, Default)]
pub struct Report {
	errors: Vec<(Path, Error)>,
}

impl Report {
	pub fn push(&mut self, path: Path, error: Error) {
		self.errors.push((path, error));
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.errors.is_empty()
	}
}

#[cfg(feature = "serde")]
impl serde::Serialize for Report {
	fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		todo!()
	}
}
