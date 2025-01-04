//! Rule for semantic versioning validation.
//!
//! See [`SemverRule`] for more information.

use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule = SemverRule;

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
			.map_err(|_| crate::error::Error::Semver)?;

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
