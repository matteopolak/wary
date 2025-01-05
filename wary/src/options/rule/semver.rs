//! Rule for semantic versioning validation.
//!
//! See [`SemverRule`] for more information.

use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule = SemverRule;

#[derive(Debug, thiserror::Error, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case", tag = "code"))]
pub enum Error {
	#[error("expected semantic version")]
	Semver,
}

impl Error {
	#[must_use]
	pub fn code(&self) -> &'static str {
		match self {
			Self::Semver => "semver",
		}
	}

	#[cfg(feature = "alloc")]
	#[must_use]
	pub fn message(&self) -> Cow<'static, str> {
		match self {
			Self::Semver => "expected semantic version".into(),
		}
	}

	#[cfg(not(feature = "alloc"))]
	pub fn message(&self) -> &'static str {
		match self {
			Self::Semver => "expected semantic version",
		}
	}
}

/// Rule for semantic versioning validation.
///
/// # Example
///
/// ```
/// use wary::{Wary, Validate};
///
/// #[derive(Wary)]
/// struct Version {
///   #[validate(semver)]
///   version: &'static str,
/// }
///
/// let version = Version { version: "1.2.3" };
/// assert!(version.validate(&()).is_ok());
///
/// let version = Version { version: "1.2.3-alpha" };
/// assert!(version.validate(&()).is_ok());
///
/// let version = Version { version: "blah" };
/// assert!(version.validate(&()).is_err());
/// ```
#[must_use]
pub struct SemverRule;

impl SemverRule {
	#[inline]
	pub const fn new() -> Self {
		Self
	}
}

impl<I: ?Sized> crate::Rule<I> for SemverRule
where
	I: AsRef<str>,
{
	type Context = ();

	#[inline]
	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let version = item.as_ref();

		// TODO: https://github.com/dtolnay/semver/issues/326
		version
			.parse::<semver::Version>()
			.map_err(|_| Error::Semver)?;

		Ok(())
	}
}

mod test {
	use crate::toolbox::test::*;

	#[derive(Wary)]
	#[wary(crate = "crate")]
	struct Version(#[validate(semver)] &'static str);

	#[test]
	fn test_semver_rule() {
		let version = Version("1.2.3");

		assert!(version.validate(&()).is_ok());

		let version = Version("1.2.3-alpha");

		assert!(version.validate(&()).is_ok());

		let version = Version("blah");

		assert!(version.validate(&()).is_err());
	}
}
