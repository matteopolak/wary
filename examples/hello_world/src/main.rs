use wary::{Transform, Wary};

#[derive(Wary, Debug)]
struct Item {
	#[transform(inner(lowercase))]
	name: Vec<String>,
}

fn main() {
	let mut item = Item {
		name: vec!["Hello".into(), "World".into()],
	};

	item.transform(&());

	assert_eq!(item.name, vec!["hello", "world"]);
}
