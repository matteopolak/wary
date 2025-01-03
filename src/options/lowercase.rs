use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule<Mode> = Lowercase<Mode>;
#[doc(hidden)]
pub type Modifier<Mode> = Lowercase<Mode>;

pub struct Lowercase<Mode> {
	mode: PhantomData<Mode>,
}

pub struct Ascii;

impl Lowercase<Unset> {
	#[must_use]
	pub fn new() -> Self {
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
	#[must_use]
	pub fn ascii(self) -> Lowercase<Ascii> {
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
			if !ch.is_lowercase() {
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
			if !ch.is_ascii_lowercase() {
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

impl crate::Modifier<String> for Lowercase<Ascii> {
	type Context = ();

	#[inline]
	fn modify(&self, _ctx: &Self::Context, item: &mut String) {
		item.make_ascii_lowercase();
	}
}
