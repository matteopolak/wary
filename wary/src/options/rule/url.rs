//! Rule for URL validation.
//!
//! See [`UrlRule`] for more information.

use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule = UrlRule;

/// Rule for URL validation.
///
/// # Example
///
/// ```
/// use wary::{Wary, Validate};
///
/// #[derive(Wary)]
/// struct Image {
///   label: String,
///   #[validate(url)]
///   src: String,
/// }
///
/// let image = Image {
///   label: "My cat".into(),
///   src: "https://example.com/cat.jpg".into(),
/// };
///
/// assert!(image.validate(&()).is_ok());
///
/// let image = Image {
///   label: "My cat".into(),
///   src: "hello".into(),
/// };
///
/// assert!(image.validate(&()).is_err());
/// ```
pub struct UrlRule;

impl UrlRule {
	#[must_use]
	#[inline]
	pub const fn new() -> Self {
		Self
	}
}

impl<I: ?Sized> crate::Rule<I> for UrlRule
where
	I: AsRef<str>,
{
	type Context = ();

	#[inline]
	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		url::Url::parse(item.as_ref())?;
		Ok(())
	}
}

#[cfg(test)]
mod test {
	use super::UrlRule;
	use crate::toolbox::test::*;

	const rule: UrlRule = UrlRule::new();

	#[test]
	fn test_url() {
		assert!(rule.validate(&(), "https://example.com").is_ok());
		assert!(rule.validate(&(), "hello").is_err());
	}
}
