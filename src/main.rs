use std::borrow::Cow;

use wary::Validate;
/*
pub struct Hi {
	min: usize,
}

fn custom(ctx: &Hi, value: &str) -> Result<(), wary::Error> {
	Ok(())
}

const LOW: &str = "hello";

#[derive(Validate)]
#[validate(context = "Hi")]
pub struct Hello<'s> {
	#[validate(length(chars, ctx.min..=10))]
	pub hello: String,

	#[validate(
		range(LOW.."world"), func = custom,
	)]
	pub age: Option<Cow<'s, str>>,
}*/

#[derive(Validate)]
pub enum HelloEnum {
	Hi {
		#[validate(contains(any_order, item = "hdx"))]
		stuff: String,
	},
	Other,
}

fn main() {
	HelloEnum::Hi { stuff: "hello world".into() }.validate(&()).unwrap();

	println!("Hello, world!");
}
