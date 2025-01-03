//! Built-in to the derive macro since
//! [`Validate::validate`](crate::Validate::validate) would need to take `&mut
//! self` as receiver in order to modify the associated
//! [`Report`](crate::error::Report).

#[cfg(test)]
mod test {
	use crate::toolbox::test::*;

	#[derive(Wary)]
	#[wary(crate = "crate")]
	struct Item {
		#[validate(ascii)]
		name: &'static str,
	}

	#[test]
	fn test_inner_rule() {
		#[derive(Wary)]
		#[wary(crate = "crate")]
		struct Container {
			#[validate(inner(dive))]
			items: Vec<Item>,
		}

		let container = Container {
			items: vec![Item { name: "Hello" }, Item { name: "world" }],
		};

		assert!(container.validate(&()).is_ok());

		let container = Container {
			items: vec![Item { name: "Hello" }, Item { name: "😃" }],
		};

		assert!(container.validate(&()).is_err());
	}

	#[test]
	fn test_inner_rule_nested() {
		#[derive(Wary)]
		#[wary(crate = "crate")]
		struct Container {
			#[validate(inner(inner(dive)))]
			items: Vec<Vec<Item>>,
		}

		let container = Container {
			items: vec![vec![Item { name: "Hello" }, Item { name: "world" }]],
		};

		assert!(container.validate(&()).is_ok());

		let container = Container {
			items: vec![vec![Item { name: "Hello" }, Item { name: "😃" }]],
		};

		assert!(container.validate(&()).is_err());
	}
}
