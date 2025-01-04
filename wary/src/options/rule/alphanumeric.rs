//! Rule for alphanumeric validation.
//!
//! See [`AlphanumericRule`] for more information.

use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule<Mode> = AlphanumericRule<Mode>;

pub struct Ascii;

/// Rule for alphanumeric validation.
///
/// # Example
///
/// ```
/// use wary::{Wary, Validate};
///
/// #[derive(Wary)]
/// struct Person {
///   #[validate(alphanumeric)]
///   name: String,
/// }
///
/// let person = Person {
///   name: "hello123".into(),
/// };
///
/// assert!(person.validate(&()).is_ok());
///
/// let person = Person {
///   name: "hello world".into(),
/// };
///
/// assert!(person.validate(&()).is_err());
/// ```
#[must_use]
pub struct AlphanumericRule<Mode> {
	mode: PhantomData<Mode>,
}

impl AlphanumericRule<Unset> {
	#[inline]
	pub const fn new() -> Self {
		Self { mode: PhantomData }
	}

	/// # Rule
	///
	/// Ensures that the input is entirely alphanumeric in ascii.
	#[inline]
	pub const fn ascii(self) -> AlphanumericRule<Ascii> {
		AlphanumericRule { mode: PhantomData }
	}
}

impl<I> crate::Rule<I> for AlphanumericRule<Unset>
where
	I: AsRef<str>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let string = item.as_ref();

		if string.chars().all(char::is_alphanumeric) {
			Ok(())
		} else {
			Err(Error::Alphanumeric)
		}
	}
}

impl<I> crate::Rule<I> for AlphanumericRule<Ascii>
where
	I: AsRef<str>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let string = item.as_ref();

		if string.chars().all(|ch| ch.is_ascii_alphanumeric()) {
			Ok(())
		} else {
			Err(Error::Alphanumeric)
		}
	}
}

#[cfg(test)]
mod test {
	use crate::toolbox::test::*;

	#[derive(Wary)]
	#[wary(crate = "crate")]
	struct Person<'name> {
		#[validate(alphanumeric)]
		name: Cow<'name, str>,
	}

	#[test]
	fn test_alphanumeric_rule() {
		let person = Person {
			name: Cow::Borrowed("Hello"),
		};

		assert!(person.validate(&()).is_ok());

		let person = Person {
			name: Cow::Borrowed("hello world"),
		};

		assert!(person.validate(&()).is_err());
	}
}
