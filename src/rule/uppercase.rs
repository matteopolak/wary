use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule_ = UppercaseRule;

pub struct UppercaseRule;

impl<I: ?Sized> Rule<I> for UppercaseRule
where
	I: AsRef<str>,
{
	type Context = ();

	#[inline]
	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<(), Error> {
		let email = item.as_ref();

		if email.is_ascii() {
			Ok(())
		} else {
			Err(Error::Ascii)
		}
	}
}
