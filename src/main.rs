use std::borrow::Cow;

use wary::Validate;

pub struct Hi {
	min: usize,
}

fn custom(ctx: &Hi, value: &str) -> Result<(), wary::Error> {
	Ok(())
}

struct fun<'d, T> {
	value: &'d T,
	fail: bool,
}

impl<'d, T> fun<'d, T> {
	fn new(value: &'d T) -> Self {
		Self { value, fail: false }
	}

	fn fail(mut self) -> Self {
		self.fail = true;
		self
	}
}

impl<'d, T> Validate for fun<'d, T> {
	type Context = ();

	fn validate(&self, ctx: &Self::Context) -> Result<(), wary::Error> {
		if self.fail {
			Err(wary::Error::Custom("lol".into()))
		} else {
			Ok(())
		}
	}
}

#[derive(Validate)]
#[validate(context = "Hi")]
pub struct Hello<'s> {
	#[validate(length(chars, min = ctx.min, max = 10))]
	pub hello: String,

	#[validate(
		range(min = "hello", max = "world"), func = custom,
	)]
	pub age: Option<Cow<'s, str>>,
}

#[derive(Validate)]
pub enum HelloEnum {
	Hi {
		#[validate(
			custom(fun(fail)),
			or(range(min = 1, max = 10), range(min = 20, max = 30))
		)]
		dog: Option<u32>,
	},
	Other,
}

fn main() {
	HelloEnum::Hi { dog: Some(10) }.validate(&()).unwrap();

	println!("Hello, world!");
}
