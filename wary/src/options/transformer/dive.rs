#[cfg(test)]
mod test {
	use crate::toolbox::test::*;

	#[test]
	fn test_dive_transformer() {
		#[derive(Wary)]
		#[wary(crate = "crate")]
		struct Item {
			#[transform(lowercase)]
			name: String,
		}

		#[derive(Wary)]
		#[wary(crate = "crate")]
		struct Name {
			#[transform(dive)]
			item: Item,
		}

		let mut name = Name {
			item: Item {
				name: "Hello".into(),
			},
		};

		name.transform(&());

		assert_eq!(name.item.name, "hello");
	}
}
