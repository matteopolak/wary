use wary::{Modify, Wary};

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

	assert_eq!(item.name, vec!["hello", "world"]);
}
