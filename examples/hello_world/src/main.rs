use wary::Wary;

#[derive(Debug, Wary)]
pub struct MyString {
	#[validate(length(bytes, 1..=5), lowercase(ascii))]
	thing: String,
}

impl AsRef<str> for MyString {
	fn as_ref(&self) -> &str {
		&self.thing
	}
}

fn testy(ctx: &(), hello: &HelloEnum) -> Result<(), wary::Error> {
	if hello.stuff.thing.contains("hello") {
		Err(wary::Error::with_message(
			"contains_hello",
			format!("found hello in {}", hello.stuff.thing),
		))
	} else if hello.stuff.thing.contains("world") {
		Err(wary::Error::new("contains_world"))
	} else {
		Ok(())
	}
}

fn test2(ctx: &(), hello: &mut HelloEnum) {
	hello.stuff.thing.push_str("world");
}

#[derive(Debug, Wary)]
#[modify(func = test2)]
pub struct HelloEnum {
	stuff: MyString,
}

fn main() {
	let mut hello = HelloEnum {
		stuff: MyString {
			thing: "hellosjhsdlkjfJshdf".into(),
		},
	};

	hello.analyze(&()).unwrap();

	println!("{:?}", hello);
}
