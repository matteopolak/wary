#[cfg(test)]
mod test {
	use crate::toolbox::test::*;

	struct SecretRule;

	impl SecretRule {
		const fn new() -> Self {
			Self
		}
	}

	impl<I> Rule<I> for SecretRule
	where
		I: AsRef<str>,
	{
		type Context = ();

		fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
			let string = item.as_ref();

			if string.contains("secret") {
				Err(Error::with_message(
					"secret_found",
					"You cannot use the word 'secret'",
				))
			} else {
				Ok(())
			}
		}
	}

	#[allow(non_camel_case_types)]
	mod rule {
		pub type secret = super::SecretRule;
	}

	#[test]
	fn test_custom_rule() {
		#[derive(Wary)]
		#[wary(crate = "crate")]
		struct Person {
			#[validate(custom(secret))]
			name: String,
		}

		let person = Person {
			name: "hello".into(),
		};

		assert!(person.validate(&()).is_ok());

		let person = Person {
			name: "secret world".into(),
		};

		assert!(person.validate(&()).is_err());
	}
}
