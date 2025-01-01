use wary::{Modify, Validate, Wary};

#[derive(Wary)]
pub struct HelloEnum {
	#[validate(length(graphemes, 1..=10))]
	#[modify(uppercase)]
	stuff: String,
}

fn main() {
	let mut hello = HelloEnum {
		stuff: "hello".into(),
	};

	hello.analyze(&()).unwrap();

	println!("{}", hello.stuff);
}
