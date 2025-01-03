use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule<Mode> = Uppercase<Mode>;
#[doc(hidden)]
pub type Modifier<Mode> = Uppercase<Mode>;

pub struct Uppercase<Mode> {
	mode: PhantomData<Mode>,
}

pub struct Ascii;

impl Uppercase<Unset> {
	#[must_use]
	pub fn new() -> Self {
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
	#[must_use]
	pub fn ascii(self) -> Uppercase<Ascii> {
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
			if !ch.is_uppercase() {
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
			if !ch.is_ascii_uppercase() {
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
