#[cfg(test)]
mod test {
	use crate::toolbox::test::*;

	struct SecretTransformer;

	impl SecretTransformer {
		const fn new() -> Self {
			Self
		}
	}

	impl AsyncTransformer<String> for SecretTransformer {
		type Context = ();

		async fn transform_async(&self, _ctx: &Self::Context, item: &mut String) {
			item.clear();
			item.push_str("secret");
		}
	}

	#[allow(non_camel_case_types)]
	mod transformer {
		pub type secret = super::SecretTransformer;
	}

	#[pollster::test]
	async fn test_custom_transformer() {
		#[derive(Wary)]
		#[wary(crate = "crate")]
		struct Person {
			#[transform(custom_async(secret))]
			name: String,
		}

		let mut person = Person {
			name: "hello".into(),
		};

		person.transform_async(&()).await;

		assert_eq!(person.name, "secret");
	}
}
