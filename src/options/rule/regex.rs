#[doc(hidden)]
pub use regex::Regex;

use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule<M> = RegexRule<M>;

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
	#[error("value does not match pattern {pattern}")]
	NoMatch { pattern: &'static str },
}

pub struct RegexRule<M> {
	matcher: M,
}

impl RegexRule<Unset> {
	#[must_use]
	pub fn new() -> Self {
		Self { matcher: Unset }
	}

	#[must_use]
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
