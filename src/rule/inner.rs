use crate::{Error, Validate};

#[doc(hidden)]
pub type Rule<'d, T, F> = InnerRule<'d, T, F>;

pub struct InnerRule<'d, T, F> {
	inner: &'d [T],
	validate: F,
}

impl<'d, T, F> InnerRule<'d, T, F> {
	pub fn new(inner: &'d [T], validate: F) -> Self
	where F: Fn(&T) -> Result<(), Error>,
	{
		Self { inner, validate }
	}
}

impl<T, F> Validate for InnerRule<'_, T, F>
where
	F: Fn(&T) -> Result<(), Error>,
{
	type Context = ();

	fn validate(&self, _ctx: &Self::Context) -> Result<(), Error> {
		for item in self.inner {
			(self.validate)(item)?;
		}

		Ok(())
	}
}
