use core::str::FromStr;

use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule = EmailRule;

#[must_use]
pub struct EmailRule;

impl EmailRule {
	#[inline]
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
	use crate::toolbox::test::*;

	#[derive(Wary)]
	#[wary(crate = "crate")]
	struct Person {
		#[validate(email)]
		email: String,
	}

	#[test]
	fn test_email_rule() {
		let person = Person {
			email: "hello@email.com".into(),
		};

		assert!(person.validate(&()).is_ok());

		let person = Person {
			email: "hello".into(),
		};

		assert!(person.validate(&()).is_err());
	}
}
