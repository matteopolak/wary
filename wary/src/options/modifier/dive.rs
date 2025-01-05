#[cfg(test)]
mod test {
	use crate::toolbox::test::*;

	#[test]
	fn test_dive_modifier() {
		#[derive(Wary)]
		#[wary(crate = "crate")]
		struct Item {
			#[modify(lowercase)]
			name: String,
		}

		#[derive(Wary)]
		#[wary(crate = "crate")]
		struct Name {
			#[modify(dive)]
			item: Item,
		}

		let mut name = Name {
			item: Item {
				name: "Hello".into(),
			},
		};

		name.modify(&());

		assert_eq!(name.item.name, "hello");
	}
}
