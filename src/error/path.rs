use std::sync::Arc;

/// A non-empty singly-linked list with O(1) append and [`Clone`].
#[derive(Clone)]
pub struct Path {
	head: Arc<Node>,
	tail: Arc<Node>,
}

// path1: root
// path2: root + hello
// path2: root + bye
// path3: root + hello + world
// path3: root + hello + universe

impl Path {
	pub fn new<E: Into<Elem>>(elem: E) -> Self {
		let node = Arc::new(Node::new(elem));

		Self {
			tail: Arc::clone(&node),
			head: node,
		}
	}

	pub fn append<E: Into<Elem>>(&self, elem: E) -> Self {
		Self {
			tail: Arc::new(Node {
				elem: elem.into(),
				prev: Some(Arc::clone(&self.tail)),
			}),
			head: Arc::clone(&self.head),
		}
	}

	pub fn prepend<E: Into<Elem>>(&self, elem: E) -> Self {
		Self {
			tail: Arc::clone(&self.tail),
			head: Arc::new(Node {
				elem: elem.into(),
				prev: Some(Arc::clone(&self.head)),
			}),
		}
	}
}

pub struct Iter<'l> {
	next: Option<&'l Node>,
}

impl<'l> Iterator for Iter<'l> {
	type Item = &'l Elem;

	#[allow(clippy::print_stdout)]
	fn next(&mut self) -> Option<Self::Item> {
		let node = self.next?;

		self.next = node.prev.as_deref();

		Some(&node.elem)
	}
}

impl<'l> IntoIterator for &'l Path {
	type IntoIter = Iter<'l>;
	type Item = &'l Elem;

	fn into_iter(self) -> Self::IntoIter {
		Iter {
			next: Some(&self.tail),
		}
	}
}

#[derive(Debug)]
pub enum Elem {
	Key(&'static str),
	Index(usize),
}

impl From<&'static str> for Elem {
	fn from(key: &'static str) -> Self {
		Self::Key(key)
	}
}

impl From<usize> for Elem {
	fn from(index: usize) -> Self {
		Self::Index(index)
	}
}

#[derive(Debug)]
pub struct Node {
	elem: Elem,
	prev: Option<Arc<Node>>,
}

impl Node {
	pub fn new<E: Into<Elem>>(elem: E) -> Self {
		Self {
			elem: elem.into(),
			prev: None,
		}
	}
}

#[cfg(all(test, not(test)))]
mod tests {
	use super::*;

	#[test]
	fn path() {
		let mut path = Path::new("a");
		let mut path = path.append("b");
		let mut path = path.append("c");

		let mut other = Path::new("1");
		let mut other = other.append("2");
		let other = other.append("3");

		path.prefix(&other);

		let mut new = Path::new("x");
		let mut new = new.append("y");
		let mut new = new.append("z");

		new.prefix(&path);

		let vec = new.into_iter().collect::<Vec<_>>();

		assert_eq!(vec, vec!["z", "y", "x", "c", "b", "a", "3", "2", "1"]);
	}

	#[test]
	fn dry_run() {
		fn run1() -> Path {
			let path = Path::new("age");
			path
		}

		fn run2() -> Path {
			let path = Path::new("person");
			let mut age = run1();

			age.prefix(&path);
			age
		}
	}
}
