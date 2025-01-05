#[cfg(test)]
mod test {
	use crate::toolbox::test::*;

	#[test]
	fn test_and_rule() {
		#[derive(Wary)]
		#[wary(crate = "crate")]
		struct Item {
			#[validate(and(alphanumeric, ascii))]
			name: &'static str,
		}

		let item = Item { name: "Hello" };

		assert!(item.validate(&()).is_ok());

		let item = Item { name: "😃" };
		let report = item.validate(&()).unwrap_err();

		assert_eq!(report.len(), 1);
	}
}
