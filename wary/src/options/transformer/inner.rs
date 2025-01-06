#[cfg(test)]
mod test {
	use crate::toolbox::test::*;

	#[test]
	fn test_inner_transformer() {
		#[derive(Wary)]
		#[wary(crate = "crate")]
		struct Item {
			#[transform(inner(lowercase))]
			name: Vec<String>,
		}

		let mut item = Item {
			name: vec!["Hello".into(), "World".into()],
		};

		item.transform(&());

		assert_eq!(item.name, vec!["hello", "world"]);
	}
}
