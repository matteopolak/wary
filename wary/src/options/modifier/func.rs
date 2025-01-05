#[cfg(test)]
mod test {
	use crate::toolbox::test::*;

	#[test]
	fn test_func_modifier() {
		#[allow(clippy::trivially_copy_pass_by_ref)]
		fn change(_ctx: &(), name: &mut String) {
			name.clear();
			name.push_str("hello, world!");
		}

		#[derive(Wary)]
		#[wary(crate = "crate")]
		struct Name {
			#[modify(func = |ctx: &(), name: &mut String| {
				name.clear();
				name.push_str("hello, world!");
			})]
			left: String,
			#[modify(func = change)]
			right: String,
		}

		let mut name = Name {
			left: "HelloWorld".into(),
			right: "HelloWorld".into(),
		};

		name.modify(&());

		assert_eq!(name.left, "hello, world!");
		assert_eq!(name.right, "hello, world!");
	}
}
