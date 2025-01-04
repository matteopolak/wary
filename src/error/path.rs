use core::fmt;

use crate::alloc::{sync::Arc, vec::Vec};

/// A non-empty singly-linked list with O(1) append and [`Clone`].
#[derive(Clone, Default)]
pub enum Path {
	#[default]
	Empty,
	NonEmpty {
		head: Arc<Node>,
		tail: Arc<Node>,
	},
}

impl fmt::Debug for Path {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut elems = self.clone().collect().into_iter();

		if let Some(elem) = elems.next() {
			write!(f, "{elem}")?;
		}

		for elem in elems {
			write!(f, ".{elem}")?;
		}

		Ok(())
	}
}

impl Path {
	pub fn new<E: Into<Elem>>(elem: E) -> Self {
		let node = Arc::new(Node::new(elem));

		Self::NonEmpty {
			tail: Arc::clone(&node),
			head: node,
		}
	}

	#[must_use]
	pub fn append<E: Into<Elem>>(&self, elem: E) -> Self {
		let Self::NonEmpty { tail, head } = self else {
			return Self::new(elem);
		};

		Self::NonEmpty {
			tail: Arc::new(Node {
				elem: elem.into(),
				prev: Some(Arc::clone(tail)),
			}),
			head: Arc::clone(head),
		}
	}

	// TODO: use smallvec or something if this is too slow
	/// Collects the path (and reverses it so it's "in order").
	#[must_use]
	pub fn collect(self) -> Vec<Elem> {
		let mut elems = self.into_iter().collect::<Vec<_>>();
		// the iterator iterates in reverse
		elems.reverse();
		elems
	}

	#[must_use]
	pub fn iter(&self) -> Iter<'_> {
		self.into_iter()
	}
}

pub struct Iter<'l> {
	next: Option<&'l Node>,
}

impl Iterator for Iter<'_> {
	type Item = Elem;

	#[allow(clippy::print_stdout)]
	fn next(&mut self) -> Option<Self::Item> {
		let node = self.next?;

		self.next = node.prev.as_deref();

		Some(node.elem)
	}
}

impl<'l> IntoIterator for &'l Path {
	type IntoIter = Iter<'l>;
	type Item = Elem;

	fn into_iter(self) -> Self::IntoIter {
		Iter {
			next: match self {
				Path::NonEmpty { tail, .. } => Some(tail),
				Path::Empty => None,
			},
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Elem {
	Key(&'static str),
	Index(usize),
}

impl fmt::Display for Elem {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Key(key) => write!(f, "{key}"),
			Self::Index(index) => write!(f, "{index}"),
		}
	}
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

#[cfg(test)]
mod test {
	use super::*;
	use crate::toolbox::test::*;

	#[test]
	fn test_path_append() {
		let path = Path::new("a").append(1).append("b").append("c");
		let vec = path.collect();

		assert_eq!(vec, vec![
			Elem::Key("a"),
			Elem::Index(1),
			Elem::Key("b"),
			Elem::Key("c"),
		]);
	}
}
