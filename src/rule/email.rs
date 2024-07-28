use std::{borrow::Cow, str::FromStr};

use crate::{Error, Validate};

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
		// TODO: remove this allocation!!!!
		email_address::EmailAddress::from_str(self.inner.email())?;
		Ok(())
	}
}

impl<T> Validate for EmailRule<Option<T>>
where
	T: Email,
{
	type Context = ();

	fn validate(&self, ctx: &Self::Context) -> Result<(), Error> {
		if let Some(inner) = &self.inner {
			EmailRule::new(inner).validate(ctx)
		} else {
			Ok(())
		}
	}
}

impl<T> Email for &T
where
	T: Email,
{
	fn email(&self) -> &str {
		(**self).email()
	}
}

impl Email for str {
	fn email(&self) -> &str {
		self
	}
}

impl Email for String {
	fn email(&self) -> &str {
		self
	}
}

impl Email for Cow<'_, str> {
	fn email(&self) -> &str {
		self
	}
}

impl Email for email_address::EmailAddress {
	fn email(&self) -> &str {
		self.as_str()
	}
}
