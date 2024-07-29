use std::borrow::Cow;

use wary::Validate;

pub struct Hi {
	min: usize,
}

fn custom(ctx: &Hi, value: &Option<Cow<str>>) -> Result<(), wary::Error> {
	Ok(())
}

#[derive(Validate)]
#[validate(context = "Hi")]
pub struct Hello<'s> {
	#[validate(length(chars, min = ctx.min, max = 10))]
	pub hello: String,

	#[validate(range(min = "hello", max = "world"), func = custom)]
	pub age: Option<Cow<'s, str>>,
}

#[derive(Validate)]
pub enum HelloEnum {
	Hi {
		#[validate(range(min = 1, max = 10))]
		dog: Option<u32>,
	},
	Other,
}

fn main() {
	HelloEnum::Hi { dog: Some(11) }.validate(&()).unwrap();

	println!("Hello, world!");
}
