#[cfg(test)]
mod test {
	use crate::toolbox::test::*;

	struct SecretModifier;

	impl SecretModifier {
		const fn new() -> Self {
			Self
		}
	}

	impl Modifier<String> for SecretModifier {
		type Context = ();

		fn modify(&self, _ctx: &Self::Context, item: &mut String) {
			item.clear();
			item.push_str("secret");
		}
	}

	#[allow(non_camel_case_types)]
	mod modifier {
		pub type secret = super::SecretModifier;
	}

	#[test]
	fn test_custom_modifier() {
		#[derive(Wary)]
		#[wary(crate = "crate")]
		struct Person {
			#[modify(custom(secret))]
			name: String,
		}

		let mut person = Person {
			name: "hello".into(),
		};

		person.modify(&());

		assert_eq!(person.name, "secret");
	}
}
