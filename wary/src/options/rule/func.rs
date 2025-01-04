#[cfg(test)]
mod test {
	use crate::toolbox::test::*;

	#[test]
	fn test_func_rule() {
		#[allow(clippy::trivially_copy_pass_by_ref)]
		fn check(_ctx: &(), name: &str) -> Result<()> {
			if name.len() > 5 {
				Ok(())
			} else {
				Err(Error::with_message(
					"name_too_short",
					"Your name must be longer than 5 characters",
				))
			}
		}

		#[derive(Wary)]
		#[wary(crate = "crate")]
		struct Name {
			#[validate(func = |ctx: &(), name: &str| {
				if name.len() > 5 {
					Ok(())
				} else {
					Err(Error::with_message("name_too_short", "Your name must be longer than 5 characters"))
				}
			})]
			left: String,
			#[validate(func = check)]
			right: String,
		}

		let name = Name {
			left: "HelloWorld".into(),
			right: "HelloWorld".into(),
		};

		assert!(name.validate(&()).is_ok());

		let name = Name {
			left: "Hi".into(),
			right: "Hi".into(),
		};

		assert!(name.validate(&()).is_err());
	}
}
