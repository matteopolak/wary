use crate::toolbox::rule::*;

#[doc(hidden)]
pub type Rule_ = UrlRule;

pub struct UrlRule;

impl UrlRule {
	pub fn new() -> Self {
		Self
	}
}

impl<I: ?Sized> Rule<I> for UrlRule
where
	I: AsRef<str>,
{
	type Context = ();

	#[inline]
	fn validate(&self, _ctx: &Self::Context, item: &I) -> Result<(), Error> {
		url::Url::parse(item.as_ref())?;
		Ok(())
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_url() {
		let url = UrlRule::new();
		assert!(url.validate(&(), "https://example.com").is_ok());

		let url = UrlRule::new();
		assert!(url.validate(&(), "hello").is_err());
	}
}
