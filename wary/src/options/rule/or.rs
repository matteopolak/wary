#[cfg(test)]
mod test {
	use crate::toolbox::test::*;

	#[test]
	fn test_or_rule() {
		#[derive(Wary)]
		#[wary(crate = "crate")]
		struct Item {
			#[validate(or(equals(other = 1), equals(other = 2)))]
			name: u32,
		}

		let item = Item { name: 1 };
		assert!(item.validate(&()).is_ok());

		let item = Item { name: 2 };
		assert!(item.validate(&()).is_ok());

		let item = Item { name: 3 };
		assert!(item.validate(&()).is_err());
	}
}
