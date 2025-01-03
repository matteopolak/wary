use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule = AlphanumericRule;

pub struct AlphanumericRule;

impl AlphanumericRule {
	#[must_use]
	pub fn new() -> Self {
		Self
	}
}

impl<I> crate::Rule<I> for AlphanumericRule
where
	I: AsRef<str>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let email = item.as_ref();

		if email.chars().all(char::is_alphanumeric) {
			Ok(())
		} else {
			Err(Error::Alphanumeric)
		}
	}
}

#[cfg(test)]
mod test {
	use std::borrow::Cow;

	use crate::toolbox::test::*;

	#[derive(Wary)]
	#[wary(crate = "crate")]
	struct Person<'name> {
		#[validate(alphanumeric)]
		name: Cow<'name, str>,
	}

	#[test]
	fn test_alphanumeric_rule() {
		let person = Person {
			name: Cow::Borrowed("Hello"),
		};

		assert!(person.validate(&()).is_ok());

		let person = Person {
			name: Cow::Borrowed("hello world"),
		};

		assert!(person.validate(&()).is_err());
	}
}
