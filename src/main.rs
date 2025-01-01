use wary::Validate;

#[derive(Validate)]
pub enum HelloEnum {
	Hi {
		#[validate(length(graphemes, 1..=10))]
		stuff: String,
	},
	Other,
}

fn main() {
	HelloEnum::Hi {
		stuff: "he6".into(),
	}
	.validate(&())
	.unwrap();

	println!("Hello, world!");
}
