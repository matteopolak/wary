use crate::{Error, Validate};

use super::Unset;

#[doc(hidden)]
pub type Rule<T, M> = MatchesRule<T, M>;

pub struct MatchesRule<T, M> {
	inner: T,
	matcher: M,
}

impl<T> MatchesRule<T, Unset> {
	pub fn new(inner: T) -> Self
	where T: AsRef<str>
 {
		Self {
			inner,
			matcher: Unset,
		}
	}

	pub fn pat(self, regex: &Regex) -> MatchesRule<T, &Regex> {
		MatchesRule {
			inner: self.inner,
			matcher: regex,
		}
	}
}

impl<T> Validate for MatchesRule<T, &'_ Regex>
	where T: AsRef<str>
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context) -> Result<(), Error> {
		if self.matcher.is_match(self.inner.as_ref()) {
			Ok(())
		} else {
			panic!()
		}
	}
}

#[doc(hidden)]
pub use regex::Regex;

