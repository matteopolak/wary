use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule<M> = MatchesRule<M>;

pub struct MatchesRule<M> {
	matcher: M,
}

impl MatchesRule<Unset> {
	pub fn new() -> Self {
		Self { matcher: Unset }
	}

	pub fn pat(self, regex: &Regex) -> MatchesRule<&Regex> {
		MatchesRule { matcher: regex }
	}
}

impl<I: ?Sized> crate::Rule<I> for MatchesRule<&'_ Regex>
where
	I: AsRef<str>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<(), Error> {
		if self.matcher.is_match(item.as_ref()) {
			Ok(())
		} else {
			panic!()
		}
	}
}

#[doc(hidden)]
pub use regex::Regex;
