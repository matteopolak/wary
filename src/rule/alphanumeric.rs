use crate::{Error, Validate};

#[doc(hidden)]
pub type Rule<T> = AlphanumericRule<T>;

pub trait Alphanumeric {
	fn alphanumeric(&self) -> &str;
}

pub struct AlphanumericRule<T> {
	inner: T,
}

impl<T> Validate for AlphanumericRule<T>
where
	T: Alphanumeric,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context) -> Result<(), Error> {
		let email = self.inner.alphanumeric();

		if email.chars().all(|c| c.is_alphanumeric()) {
			Ok(())
		} else {
			Err(Error::Alphanumeric)
		}
	}
}

impl<T> Alphanumeric for T
where
	T: AsRef<str>,
{
	fn alphanumeric(&self) -> &str {
		self.as_ref()
	}
}
