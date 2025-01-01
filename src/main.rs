use wary::Validate;

#[derive(Validate)]
pub struct HelloEnum {
	#[validate(email, length(graphemes, 1..=10))]
	stuff: String,
}

fn main() {
	HelloEnum { stuff: "".into() }.validate(&()).unwrap();

	println!("Hello, world!");
}
