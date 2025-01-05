//! Rule for validating a value against a regular expression.
//!
//! See [`RegexRule`] for more information.

#[doc(hidden)]
pub use regex::Regex;

use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule<M> = RegexRule<M>;

#[derive(Debug, thiserror::Error, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case", tag = "code"))]
pub enum Error {
	#[error("value does not match pattern {pattern}")]
	NoMatch { pattern: &'static str },
}

impl Error {
	#[must_use]
	pub fn code(&self) -> &'static str {
		match self {
			Self::NoMatch { .. } => "no_match",
		}
	}

	#[cfg(feature = "alloc")]
	#[must_use]
	pub fn message(&self) -> Cow<'static, str> {
		match self {
			Self::NoMatch { pattern } => format!("value does not match pattern {pattern}").into(),
		}
	}

	#[cfg(not(feature = "alloc"))]
	pub fn message(&self) -> &'static str {
		match self {
			Self::NoMatch { .. } => "value does not match pattern",
		}
	}
}

/// Rule for validating a value against a regular expression.
///
/// # Example
///
/// ```
/// use wary::{Wary, Validate};
///
/// #[derive(Wary)]
/// struct Text {
///   #[validate(regex(pat = "^a+"))]
///   a: &'static str,
///   #[validate(regex(pat = "^b+"))]
///   b: &'static str,
/// }
///
/// let text = Text { a: "aaa", b: "bbb" };
/// assert!(text.validate(&()).is_ok());
///
/// let text = Text { a: "aaa", b: "ccc" };
/// assert!(text.validate(&()).is_err());
/// ```
#[must_use]
pub struct RegexRule<M> {
	matcher: M,
}

impl RegexRule<Unset> {
	#[inline]
	pub const fn new() -> Self {
		Self { matcher: Unset }
	}

	/// Set the regular expression pattern to match.
	///
	/// This can either be a literal string or a compiled regular expression
	/// in some sort of static container like [`LazyCell`](std::cell::LazyCell) or
	/// [`OnceCell`](std::cell::OnceCell).
	#[inline]
	pub fn pat(self, regex: &'static Regex) -> RegexRule<&'static Regex> {
		RegexRule { matcher: regex }
	}
}

impl<I: ?Sized> crate::Rule<I> for RegexRule<&'static Regex>
where
	I: AsRef<str>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		if self.matcher.is_match(item.as_ref()) {
			Ok(())
		} else {
			Err(
				Error::NoMatch {
					pattern: self.matcher.as_str(),
				}
				.into(),
			)
		}
	}
}

mod test {
	use crate::toolbox::test::*;

	#[derive(Wary)]
	#[wary(crate = "crate")]
	struct Text {
		#[validate(regex(pat = "^a+"))]
		a: &'static str,
		#[validate(regex(pat = "^b+"))]
		b: &'static str,
	}

	#[test]
	fn test_regex_rule() {
		let text = Text { a: "aaa", b: "bbb" };

		assert!(text.validate(&()).is_ok());

		let text = Text { a: "aaa", b: "ccc" };

		assert!(text.validate(&()).is_err());
	}
}
