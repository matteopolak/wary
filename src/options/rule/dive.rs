#[cfg(test)]
mod test {
	use crate::toolbox::test::*;

	#[test]
	fn test_dive_rule() {
		#[derive(Wary)]
		#[wary(crate = "crate")]
		struct Item {
			#[validate(ascii)]
			name: &'static str,
		}

		#[derive(Wary)]
		#[wary(crate = "crate")]
		struct Name {
			#[validate(dive)]
			item: Item,
		}

		let name = Name {
			item: Item { name: "Hello" },
		};

		assert!(name.validate(&()).is_ok());

		let name = Name {
			item: Item { name: "😃" },
		};

		assert!(name.validate(&()).is_err());
	}
}
