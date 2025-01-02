use wary::{Wary};

#[derive(Wary)]
pub struct MyString {
	#[validate(length(bytes, 1..=5), lowercase(ascii))]
	thing: String
}

impl AsRef<str> for MyString {
	fn as_ref(&self) -> &str {
	  &self.thing
	}
}

#[derive(Wary)]
pub struct HelloEnum {
	#[validate(dive, regex(pat = r"efi\d"))]
	stuff: MyString,
}

fn main() {
	let mut hello = HelloEnum {
		stuff: MyString { thing: "hellosjhsdlkjfJshdf".into() },
	};

	let report = hello.analyze(&()).unwrap_err();

	println!("{:?}", report);
}
