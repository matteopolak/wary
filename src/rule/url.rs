use std::borrow::Cow;

use crate::{Error, Validate};

pub trait Url {
	fn url(&self) -> Option<&str>;
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
		let Some(url) = self.inner.url() else {
			return Ok(());
		};

		url::Url::parse(url)?;
		Ok(())
	}
}

impl<T> Url for &T
where
	T: Url,
{
	fn url(&self) -> Option<&str> {
		(**self).url()
	}
}

impl<T> Url for Option<T>
where
	T: Url,
{
	fn url(&self) -> Option<&str> {
		self.as_ref().and_then(Url::url)
	}
}

impl Url for &str {
	fn url(&self) -> Option<&str> {
		Some(self)
	}
}

impl Url for String {
	fn url(&self) -> Option<&str> {
		Some(self)
	}
}

impl Url for Cow<'_, str> {
	fn url(&self) -> Option<&str> {
		Some(self)
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_url() {
		let url = UrlRule::new("https://example.com");
		assert!(url.validate(&()).is_ok());

		let url = UrlRule::new("example.com");
		assert!(url.validate(&()).is_err());
	}
}
