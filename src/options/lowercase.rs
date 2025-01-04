//! Rule and modifier for ensuring that a string is entirely lowercase.
//!
//! See [`Lowercase`] for more information.

use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule<Mode> = Lowercase<Mode>;
#[doc(hidden)]
pub type Modifier<Mode> = Lowercase<Mode>;

pub struct Ascii;

/// Rule and modifier for ensuring that a string is entirely lowercase.
///
/// # Example
///
/// ```
/// use wary::Wary;
///
/// #[derive(Wary)]
/// struct Person {
///   #[validate(lowercase)]
///   name: String,
///   #[validate(lowercase(ascii))]
///   greeting: String,
///   #[modify(lowercase)]
///   message: String,
/// }
///
/// let mut person = Person {
///   name: "hello".into(),
///   greeting: "hello".into(),
///   message: "HELLO".into(),
/// };
///
/// assert!(person.wary(&()).is_ok());
/// assert_eq!(person.message, "hello");
/// ```
#[must_use]
pub struct Lowercase<Mode> {
	mode: PhantomData<Mode>,
}

impl Lowercase<Unset> {
	#[inline]
	pub const fn new() -> Self {
		Self { mode: PhantomData }
	}

	/// # Rule
	///
	/// Ensures that the input is entirely lowercase in ascii.
	///
	/// # Modifier
	///
	/// Uses [`str::make_ascii_lowercase`] to convert in-place instead
	/// of requiring a new allocation with [`str::to_lowercase`].
	#[inline]
	pub const fn ascii(self) -> Lowercase<Ascii> {
		Lowercase { mode: PhantomData }
	}
}

impl<I: ?Sized> crate::Rule<I> for Lowercase<Unset>
where
	I: AsRef<str>,
{
	type Context = ();

	#[inline]
	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let string = item.as_ref();

		for (idx, ch) in string.chars().enumerate() {
			if !ch.is_lowercase() && !ch.is_whitespace() {
				return Err(Error::Lowercase { position: idx });
			}
		}

		Ok(())
	}
}

impl<I: ?Sized> crate::Rule<I> for Lowercase<Ascii>
where
	I: AsRef<str>,
{
	type Context = ();

	#[inline]
	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<()> {
		let string = item.as_ref();

		for (idx, ch) in string.chars().enumerate() {
			if !ch.is_ascii_lowercase() && !ch.is_ascii_whitespace() {
				return Err(Error::Lowercase { position: idx });
			}
		}

		Ok(())
	}
}

impl crate::Modifier<String> for Lowercase<Unset> {
	type Context = ();

	#[inline]
	fn modify(&self, _ctx: &Self::Context, item: &mut String) {
		*item = item.to_lowercase();
	}
}

impl<I> crate::Modifier<I> for Lowercase<Ascii>
where
	I: AsMut<str>,
{
	type Context = ();

	#[inline]
	fn modify(&self, _ctx: &Self::Context, item: &mut I) {
		item.as_mut().make_ascii_lowercase();
	}
}

#[cfg(test)]
mod test {
	use super::Lowercase;
	use crate::toolbox::test::*;

	#[test]
	fn test_lowercase_rule() {
		let rule = Lowercase::new();
		let input = "ὈΔΥΣΣΕΎΣ hello".to_string();

		assert!(rule.validate(&(), &input).is_err());

		let rule = Lowercase::new().ascii();
		let input = "ὈΔΥΣΣΕΎΣ".to_string();

		assert!(rule.validate(&(), &input).is_err());

		let rule = Lowercase::new().ascii();
		let input = "hello world".to_string();

		assert!(rule.validate(&(), &input).is_ok());
	}

	#[test]
	fn test_lowercase_modifier() {
		let rule = Lowercase::new();
		let mut input = "ὈΔΥΣΣΕΎΣ HELLO".to_string();

		rule.modify(&(), &mut input);
		assert_eq!(input, "ὀδυσσεύς hello");

		let rule = Lowercase::new().ascii();
		let mut input = "ßeLLO".to_string();

		rule.modify(&(), &mut input);
		assert_eq!(input, "ßello");
	}
}
