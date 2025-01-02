use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule = AlphanumericRule;

pub struct AlphanumericRule;

impl<I> crate::Rule<I> for AlphanumericRule
where
	I: AsRef<str>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let email = item.as_ref();

		if email.chars().all(|c| c.is_alphanumeric()) {
			Ok(())
		} else {
			Err(Error::Alphanumeric)
		}
	}
}
