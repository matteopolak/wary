use validator::Validate;

pub struct Hi {
	min: usize,
}

fn custom(ctx: &Hi, value: &Option<String>) -> Result<(), validator::Error> {
	Ok(())
}

#[derive(Validate)]
#[validate(context = "Hi")]
pub struct Hello {
	#[validate(length(chars, min = ctx.min, max = 10))]
	pub hello: String,

	#[validate(range(min = "hello", max = "world"), func = custom)]
	pub age: Option<String>,
}

fn main() {
	Hello {
		hello: "world".to_string(),
		age: Some("hella".to_string()),
	}
	.validate(&Hi { min: 1 })
	.unwrap();

	println!("Hello, world!");
}
