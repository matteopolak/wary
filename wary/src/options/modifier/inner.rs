#[cfg(test)]
mod test {
	use crate::toolbox::test::*;

	#[test]
	fn test_inner_modifier() {
		#[derive(Wary)]
		#[wary(crate = "crate")]
		struct Item {
			#[modify(inner(lowercase))]
			name: Vec<String>,
		}

		let mut item = Item {
			name: vec!["Hello".into(), "World".into()],
		};

		item.modify(&());

		assert_eq!(item.name, vec!["hello", "world"]);
	}
}
