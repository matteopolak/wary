//! Modifier for trimming whitespace from the ends of a string.

use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Modifier<Mode> = TrimModifier<Mode>;

pub struct Ascii;

/// Modifier for trimming whitespace from the ends of a string.
///
/// # Example
///
/// ```
/// use wary::{Wary, Modify};
///
/// #[derive(Wary)]
/// struct Person {
///   #[modify(trim)]
///   message: String,
/// }
///
/// let mut person = Person {
///   message: " hello ".into(),
/// };
///
/// person.modify(&());
/// assert_eq!(person.message, "hello");
/// ```
#[must_use]
pub struct TrimModifier<Mode> {
	mode: PhantomData<Mode>,
}

impl TrimModifier<Unset> {
	#[inline]
	pub const fn new() -> Self {
		Self { mode: PhantomData }
	}

	/// Only trims ASCII whitespace.
	#[inline]
	pub const fn ascii(self) -> TrimModifier<Ascii> {
		TrimModifier { mode: PhantomData }
	}
}

impl crate::Modifier<String> for TrimModifier<Unset> {
	type Context = ();

	#[inline]
	fn modify(&self, _ctx: &Self::Context, item: &mut String) {
		let end = item.trim_end().len();

		item.truncate(end);

		let start = item.len() - item.trim_start().len();

		item.drain(..start);
	}
}

impl crate::Modifier<String> for TrimModifier<Ascii> {
	type Context = ();

	#[inline]
	fn modify(&self, _ctx: &Self::Context, item: &mut String) {
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
	use super::TrimModifier;
	use crate::toolbox::test::*;

	#[test]
	fn test_trim_modifier() {
		let rule = TrimModifier::new();
		let mut input = " hello ".to_string();

		rule.modify(&(), &mut input);
		assert_eq!(input, "hello");

		let rule = TrimModifier::new().ascii();
		let mut input = " hello ".to_string();

		rule.modify(&(), &mut input);
		assert_eq!(input, "hello");
	}
}
