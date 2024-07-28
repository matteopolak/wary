use std::borrow::Cow;

use crate::{Error, Validate};

pub trait Url {
	fn url(&self) -> &str;
}

pub struct UrlRule<T> {
	inner: T,
}

impl<T> UrlRule<T> {
	pub fn new(inner: T) -> Self {
		Self { inner }
	}
}

impl<T> Validate for UrlRule<T>
where
	T: Url,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context) -> Result<(), Error> {
		url::Url::parse(self.inner.url())?;
		Ok(())
	}
}

impl<T> Validate for UrlRule<Option<T>>
where
	T: Url,
{
	type Context = ();

	fn validate(&self, ctx: &Self::Context) -> Result<(), Error> {
		if let Some(inner) = &self.inner {
			UrlRule::new(inner).validate(ctx)
		} else {
			Ok(())
		}
	}
}

impl<T> Url for &T
where
	T: Url,
{
	fn url(&self) -> &str {
		(**self).url()
	}
}

impl Url for str {
	fn url(&self) -> &str {
		self
	}
}

impl Url for String {
	fn url(&self) -> &str {
		self
	}
}

impl Url for Cow<'_, str> {
	fn url(&self) -> &str {
		self
	}
}
