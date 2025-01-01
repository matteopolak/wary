use crate::{Error, Validate};

#[doc(hidden)]
pub type Rule<T> = SemverRule<T>;

pub struct SemverRule<T> {
	inner: T,
}

impl<T> Validate for SemverRule<T>
where
	T: AsRef<str>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context) -> Result<(), Error> {
		let version = self.inner.as_ref();

		version.parse::<semver::Version>()?;
		
		Ok(())
	}
}

