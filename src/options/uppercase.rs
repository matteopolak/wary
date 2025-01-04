//! Rule and modifier for ensuring that a string is entirely uppercase.
//!
//! See [`Uppercase`] for more information.

use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule<Mode> = Uppercase<Mode>;
#[doc(hidden)]
pub type Modifier<Mode> = Uppercase<Mode>;

pub struct Ascii;

/// Rule and modifier for ensuring that a string is entirely uppercase.
///
/// # Example
///
/// ```
/// use wary::Wary;
///
/// #[derive(Wary)]
/// struct Person {
///   #[validate(uppercase)]
///   name: String,
///   #[validate(uppercase(ascii))]
///   greeting: String,
///   #[modify(uppercase)]
///   message: String,
/// }
///
/// let mut person = Person {
///   name: "HELLO".into(),
///   greeting: "HELLO".into(),
///   message: "hello".into(),
/// };
///
/// assert!(person.wary(&()).is_ok());
/// assert_eq!(person.message, "HELLO");
/// ```
#[must_use]
pub struct Uppercase<Mode> {
	mode: PhantomData<Mode>,
}

impl Uppercase<Unset> {
	#[inline]
	pub const fn new() -> Self {
		Self { mode: PhantomData }
	}

	/// # Rule
	///
	/// Ensures that the input is entirely uppercase in ascii.
	///
	/// # Modifier
	///
	/// Uses [`str::make_ascii_uppercase`] to convert in-place instead
	/// of requiring a new allocation with [`str::to_uppercase`].
	#[inline]
	pub const fn ascii(self) -> Uppercase<Ascii> {
		Uppercase { mode: PhantomData }
	}
}

impl<I: ?Sized> crate::Rule<I> for Uppercase<Unset>
where
	I: AsRef<str>,
{
	type Context = ();

	#[inline]
	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let string = item.as_ref();

		for (idx, ch) in string.chars().enumerate() {
			if !ch.is_uppercase() && !ch.is_whitespace() {
				return Err(Error::Uppercase { position: idx });
			}
		}

		Ok(())
	}
}

impl<I: ?Sized> crate::Rule<I> for Uppercase<Ascii>
where
	I: AsRef<str>,
{
	type Context = ();

	#[inline]
	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let string = item.as_ref();

		for (idx, ch) in string.chars().enumerate() {
			if !ch.is_ascii_uppercase() && !ch.is_ascii_whitespace() {
				return Err(Error::Uppercase { position: idx });
			}
		}

		Ok(())
	}
}

impl crate::Modifier<String> for Uppercase<Unset> {
	type Context = ();

	#[inline]
	fn modify(&self, _ctx: &Self::Context, item: &mut String) {
		*item = item.to_uppercase();
	}
}

impl crate::Modifier<String> for Uppercase<Ascii> {
	type Context = ();

	#[inline]
	fn modify(&self, _ctx: &Self::Context, item: &mut String) {
		item.make_ascii_uppercase();
	}
}

#[cfg(test)]
mod test {
	use super::Uppercase;
	use crate::toolbox::test::*;

	#[test]
	fn test_uppercase_rule() {
		let rule = Uppercase::new();
		let input = "ὈΔΥΣΣΕΎΣ HELLO".to_string();

		assert!(rule.validate(&(), &input).is_ok());

		let rule = Uppercase::new().ascii();
		let input = "ὈΔΥΣΣΕΎΣ".to_string();

		assert!(rule.validate(&(), &input).is_err());

		let rule = Uppercase::new().ascii();
		let input = "HELLO WORLD".to_string();

		assert!(rule.validate(&(), &input).is_ok());
	}

	#[test]
	fn test_uppercase_modifier() {
		let rule = Uppercase::new();
		let mut input = "ὀδυσσεύς hello".to_string();

		rule.modify(&(), &mut input);
		assert_eq!(input, "ὈΔΥΣΣΕΎΣ HELLO");

		let rule = Uppercase::new().ascii();
		let mut input = "ὀδυσσεύς hello".to_string();

		rule.modify(&(), &mut input);
		assert_eq!(input, "ὀδυσσεύς HELLO");
	}
}
