use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule = SemverRule;

pub struct SemverRule;

impl SemverRule {
	pub fn new() -> Self {
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

		version.parse::<semver::Version>()?;

		Ok(())
	}
}
