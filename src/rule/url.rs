use crate::{Error, Validate};

#[doc(hidden)]
pub type Rule<T> = UrlRule<T>;

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
		let url = self.inner.url();

		url::Url::parse(url)?;
		Ok(())
	}
}

impl<T> Url for T
where
	T: AsRef<str>,
{
	fn url(&self) -> &str {
		self.as_ref()
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
