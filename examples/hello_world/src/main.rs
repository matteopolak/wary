use wary::{Modify, Validate, Wary};

#[derive(Wary, Debug)]
struct Item {
	#[modify(inner(lowercase))]
	name: Vec<String>,
}

fn main() {
	let mut item = Item {
		name: vec!["Hello".into(), "World".into()],
	};

	item.modify(&());

	println!("{:?}", item);
}
