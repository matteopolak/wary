use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule<M> = RegexRule<M>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("value does not match pattern {pattern}")]
	NoMatch { pattern: &'static str },
}

pub struct RegexRule<M> {
	matcher: M,
}

impl RegexRule<Unset> {
	pub fn new() -> Self {
		Self { matcher: Unset }
	}

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

#[doc(hidden)]
pub use regex::Regex;
