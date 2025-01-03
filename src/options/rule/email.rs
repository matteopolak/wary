use core::str::FromStr;

use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule = EmailRule;

pub struct EmailRule;

impl EmailRule {
	#[must_use]
	pub fn new() -> Self {
		Self
	}
}

impl<I: ?Sized> crate::Rule<I> for EmailRule
where
	I: AsRef<str>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let email = item.as_ref();

		// TODO: Look into avoiding the allocation
		email_address::EmailAddress::from_str(email)?;
		Ok(())
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::Rule;

	#[test]
	fn test_email() {
		let email = "hello@gmail.com";

		let rule = EmailRule::new();
		assert!(rule.validate(&(), email).is_ok());

		let rule = EmailRule::new();
		assert!(rule.validate(&(), "invalid").is_err());
	}
}
