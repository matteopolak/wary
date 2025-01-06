//! Transformer for trimming whitespace from the ends of a string.

use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Transformer<Mode> = TrimTransformer<Mode>;

pub struct Ascii;

/// Transformer for trimming whitespace from the ends of a string.
///
/// # Example
///
/// ```
/// use wary::{Wary, Transform};
///
/// #[derive(Wary)]
/// struct Person {
///   #[transform(trim)]
///   message: String,
/// }
///
/// let mut person = Person {
///   message: " hello ".into(),
/// };
///
/// person.transform(&());
/// assert_eq!(person.message, "hello");
/// ```
#[must_use]
pub struct TrimTransformer<Mode> {
	mode: PhantomData<Mode>,
}

impl TrimTransformer<Unset> {
	#[inline]
	pub const fn new() -> Self {
		Self { mode: PhantomData }
	}

	/// Only trims ASCII whitespace.
	#[inline]
	pub const fn ascii(self) -> TrimTransformer<Ascii> {
		TrimTransformer { mode: PhantomData }
	}
}

impl crate::Transformer<String> for TrimTransformer<Unset> {
	type Context = ();

	#[inline]
	fn transform(&self, _ctx: &Self::Context, item: &mut String) {
		let end = item.trim_end().len();

		item.truncate(end);

		let start = item.len() - item.trim_start().len();

		item.drain(..start);
	}
}

impl crate::Transformer<String> for TrimTransformer<Ascii> {
	type Context = ();

	#[inline]
	fn transform(&self, _ctx: &Self::Context, item: &mut String) {
		let end = item
			.trim_end_matches(|c: char| c.is_ascii_whitespace())
			.len();

		item.truncate(end);

		let start = item.len()
			- item
				.trim_start_matches(|c: char| c.is_ascii_whitespace())
				.len();

		item.drain(..start);
	}
}

#[cfg(test)]
mod test {
	use super::TrimTransformer;
	use crate::toolbox::test::*;

	#[test]
	fn test_trim_transformer() {
		let rule = TrimTransformer::new();
		let mut input = " hello ".to_string();

		rule.transform(&(), &mut input);
		assert_eq!(input, "hello");

		let rule = TrimTransformer::new().ascii();
		let mut input = " hello ".to_string();

		rule.transform(&(), &mut input);
		assert_eq!(input, "hello");
	}
}
