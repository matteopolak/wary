use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule_ = AsciiRule;

pub struct AsciiRule;

impl AsciiRule {
	pub fn new() -> Self {
		Self
	}
}

impl<I: ?Sized> Rule<I> for AsciiRule
where
	I: AsRef<str>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<(), Error> {
		let email = item.as_ref();

		if email.is_ascii() {
			Ok(())
		} else {
			Err(Error::Ascii)
		}
	}
}
