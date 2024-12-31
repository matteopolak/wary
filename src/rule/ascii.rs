use crate::{Error, Validate};

#[doc(hidden)]
pub type Rule<T> = AsciiRule<T>;

pub trait Ascii {
	fn ascii(&self) -> &str;
}

pub struct AsciiRule<T> {
	inner: T,
}

impl<T> Validate for AsciiRule<T>
where
	T: Ascii,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context) -> Result<(), Error> {
		let email = self.inner.ascii();

		if email.is_ascii() {
			Ok(())
		} else {
			Err(Error::Ascii)
		}
	}
}

impl<T> Ascii for T where T: AsRef<str> {
	fn ascii(&self) -> &str {
		self.as_ref()
	}
}
