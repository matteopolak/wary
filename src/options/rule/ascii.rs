use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule = AsciiRule;

pub struct AsciiRule;

impl AsciiRule {
	#[must_use]
	pub fn new() -> Self {
		Self
	}
}

impl<I: ?Sized> crate::Rule<I> for AsciiRule
where
	I: AsRef<str>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let email = item.as_ref();

		if email.is_ascii() {
			Ok(())
		} else {
			Err(Error::Ascii)
		}
	}
}
