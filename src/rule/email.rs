use std::{borrow::Cow, str::FromStr};

use crate::{Error, Validate};

#[doc(hidden)]
pub type Rule<T> = EmailRule<T>;

/// Used for retrieving a potentially-invalid email address.
pub trait Email {
	fn email(&self) -> &str;
}

pub struct EmailRule<T> {
	inner: T,
}

impl<T> EmailRule<T> {
	pub fn new(inner: T) -> Self {
		Self { inner }
	}
}

impl<T> Validate for EmailRule<T>
where
	T: Email,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context) -> Result<(), Error> {
		let Some(email) = self.inner.email() else {
			return Ok(());
		};

		// TODO: remove this allocation!!!!
		email_address::EmailAddress::from_str(email)?;
		Ok(())
	}
}

impl<T> Validate for EmailRule<email_address::EmailAddress> {
	type Context = ();

	fn validate(&self, _ctx: &Self::Context) -> Result<(), Error> {
		Ok(())
	}
}

impl<T> Email for T
where
	T: AsRef<str>,
{
	fn email(&self) -> &str {
		self.as_ref()
	}
}

impl<T> Email for &T
where
	T: Email,
{
	fn email(&self) -> Option<&str> {
		(**self).email()
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_email() {
		let email = "hello@gmail.com";

		let rule = EmailRule::new(email);
		assert!(rule.validate(&()).is_ok());

		let rule = EmailRule::new(Some(email));
		assert!(rule.validate(&()).is_ok());

		let rule = EmailRule::new(Some("invalid"));
		assert!(rule.validate(&()).is_err());

		let rule = EmailRule::new(None::<&str>);
		assert!(rule.validate(&()).is_ok());
	}
}